use crate::server::handler_stop::Stop;
use actix::{Actor, Addr, Running};

use crate::server::actor::ServerActor;
use crate::servers_supervisor::start_handler::{
    ChildCreated, ChildNotCreated, ChildNotPatched, ChildPatched, ChildResult,
};

use bsnext_input::server_config::Identity;
use bsnext_input::Input;
use std::collections::HashSet;
use std::future::Future;
use std::net::SocketAddr;

use crate::server::error::PatchError;
use crate::server::handler_listen::Listen;
use crate::server::handler_patch::Patch;
use futures_util::future::join_all;
use futures_util::FutureExt;
use std::pin::Pin;
use tokio::sync::oneshot::Sender;
use tracing::{span, Instrument, Level};

#[derive(Debug)]
pub struct ServersSupervisor {
    pub(crate) handlers: std::collections::HashMap<Identity, ChildHandler>,
    tx: Option<Sender<()>>,
}

#[derive(Debug, Clone)]
pub struct ChildHandler {
    pub actor_address: Addr<ServerActor>,
    pub identity: Identity,
    pub socket_addr: SocketAddr,
}

impl ServersSupervisor {
    pub fn new(tx: Sender<()>) -> Self {
        Self {
            handlers: std::default::Default::default(),
            tx: Some(tx),
        }
    }

    pub(crate) fn input_changed(
        &mut self,
        self_addr: Addr<ServersSupervisor>,
        input: Input,
    ) -> Pin<Box<impl Future<Output = Vec<ChildResult>> + Sized>> {
        let span = span!(Level::TRACE, "input_changed");
        let _guard = span.enter();

        let existing: HashSet<_> = self.handlers.values().map(|v| v.identity.clone()).collect();
        let incoming: HashSet<_> = input.servers.iter().map(|s| s.identity.clone()).collect();

        let startup = incoming.difference(&existing).collect::<HashSet<_>>();
        let shutdown = existing.difference(&incoming).collect::<HashSet<_>>();
        let patch = existing.intersection(&incoming).collect::<HashSet<_>>();

        let shutdown_jobs = shutdown
            .into_iter()
            .filter_map(|identity| self.handlers.get(identity).map(ToOwned::to_owned))
            .collect::<Vec<_>>();

        let start_jobs = startup
            .iter()
            .filter_map(|s| {
                input
                    .servers
                    .iter()
                    .find(|x| x.identity == **s)
                    .map(ToOwned::to_owned)
            })
            .collect::<Vec<_>>();

        let patch_jobs = patch
            .iter()
            .filter_map(|s| {
                let child = self.handlers.get(s).map(ToOwned::to_owned);
                let input = input
                    .servers
                    .iter()
                    .find(|x| x.identity == **s)
                    .map(ToOwned::to_owned);
                match (child, input) {
                    (Some(child), Some(config)) => Some((child, config)),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();

        Box::pin(
            async move {
                let shutdown_futs = shutdown_jobs.iter().map(|x| x.actor_address.send(Stop));
                let shutdown_results = join_all(shutdown_futs).await;
                let shutdown_child_results =
                    shutdown_results
                        .iter()
                        .zip(shutdown_jobs)
                        .map(|(r, h)| match r {
                            Ok(_) => ChildResult::Stopped(h.identity.clone()),
                            Err(er) => unreachable!("{}", er),
                        });

                tracing::debug!("starting {:?} servers", start_jobs.len());
                let fts = start_jobs.into_iter().map(|server_config| {
                    let server = ServerActor::new_from_config(server_config.clone());
                    let actor_addr = server.start();
                    let c = server_config.clone();
                    actor_addr
                        .send(Listen {
                            parent: self_addr.clone().recipient(),
                        })
                        .map(|r| (r, c))
                });
                let results = join_all(fts).await;
                let start_child_results = results.into_iter().map(|(r, c)| match r {
                    Ok(Ok((socket, addr))) => ChildResult::Created(ChildCreated {
                        server_handler: ChildHandler {
                            actor_address: addr,
                            identity: c.identity,
                            socket_addr: socket,
                        },
                    }),
                    Ok(Err(err)) => ChildResult::Err(ChildNotCreated {
                        server_error: err,
                        identity: c.identity.clone(),
                    }),
                    Err(e) => unreachable!("{}", e),
                });

                let fts = patch_jobs.into_iter().map(|(child, server_config)| {
                    child
                        .actor_address
                        .send(Patch { server_config })
                        .map(|r| (r, child))
                });
                let results = join_all(fts).await;
                let patch_child_results = results.into_iter().map(|(r, child_handler)| match r {
                    Ok(Ok(_)) => ChildResult::Patched(ChildPatched {
                        server_handler: child_handler,
                    }),
                    Ok(Err(err)) => ChildResult::PatchErr(ChildNotPatched {
                        patch_error: PatchError::DidNotPatch {
                            reason: err.to_string(),
                        },
                        identity: child_handler.identity.clone(),
                    }),
                    Err(_) => unreachable!("mailbox error on patch"),
                });

                shutdown_child_results
                    .chain(start_child_results)
                    .chain(patch_child_results)
                    .collect()
            }
            .instrument(span.clone()),
        )
    }
}

impl Actor for ServersSupervisor {
    type Context = actix::Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(actor.name = "Servers", actor.lifecyle = "started")
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        tracing::trace!(actor.name = "Servers", actor.lifecyle = "stopping");
        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        let span = span!(parent: None, Level::TRACE, "stopped", actor.name = "Servers");
        tracing::trace!(actor.lifecyle = "stopping");
        let _guard = span.enter();
        if let Some(tx) = self.tx.take() {
            tracing::trace!("sending final completion");
            match tx.send(()) {
                Ok(_) => {
                    tracing::trace!("✅ sent final completion message");
                }
                Err(_) => {
                    tracing::trace!("❌ could not send final completion message. This usually means the actor went away (eg: a crash) before we could send this.");
                }
            }
        } else {
            tracing::error!("could not access oneshot sender for completion message")
        }
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct ChildStopped {
    pub identity: Identity,
}

impl actix::Handler<ChildStopped> for ServersSupervisor {
    type Result = ();

    fn handle(&mut self, msg: ChildStopped, _ctx: &mut Self::Context) -> Self::Result {
        tracing::trace!("Handler<Stopped> for Servers {:?}", msg.identity);
        self.handlers.remove(&msg.identity);
        tracing::trace!(
            "Handler<Stopped> remaining handlers: {}",
            self.handlers.len()
        )
    }
}
