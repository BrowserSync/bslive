use crate::server::handler_stop::Stop;
use actix::{Actor, Addr, ResponseFuture, Running};

use crate::server::actor::ServerActor;

use bsnext_input::server_config::ServerIdentity;
use bsnext_input::Input;
use std::collections::HashSet;
use std::net::SocketAddr;

use crate::runtime_ctx::RuntimeCtx;
use crate::server::handler_listen::Listen;
use crate::server::handler_patch::Patch;
use crate::servers_supervisor::input_changed_handler::InputChangedResponse;
use bsnext_dto::internal::{
    ChildCreated, ChildHandlerMinimal, ChildNotCreated, ChildNotPatched, ChildPatched, ChildResult,
    PatchError,
};
use futures_util::future::join_all;
use futures_util::FutureExt;
use tokio::sync::oneshot::Sender;
use tracing::{span, Instrument, Level};

#[derive(Debug)]
pub struct ServersSupervisor {
    pub(crate) handlers: std::collections::HashMap<ServerIdentity, ChildHandler>,
    tx: Option<Sender<()>>,
}

#[derive(Debug, Clone)]
pub struct ChildHandler {
    pub actor_address: Addr<ServerActor>,
    pub identity: ServerIdentity,
    pub socket_addr: SocketAddr,
}

impl ChildHandler {
    pub fn minimal(&self) -> ChildHandlerMinimal {
        ChildHandlerMinimal {
            identity: self.identity.clone(),
            socket_addr: self.socket_addr,
        }
    }
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
    ) -> ResponseFuture<InputChangedResponse> {
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
                            Ok(_) => (None, ChildResult::Stopped(h.identity.clone())),
                            Err(er) => unreachable!("{}", er),
                        });

                tracing::debug!("starting {:?} servers", start_jobs.len());
                let startup_futures = start_jobs.into_iter().map(|server_config| {
                    let server = ServerActor::new_from_config(server_config.clone());
                    let actor_addr = server.start();
                    let actor_addr_c = actor_addr.clone();
                    let config_clone = server_config.clone();
                    actor_addr
                        .send(Listen {
                            // todo: tie this to the input somehow?
                            runtime_ctx: RuntimeCtx::default(),
                            parent: self_addr.clone().recipient(),
                            evt_receiver: self_addr.clone().recipient(),
                        })
                        .map(|listen_response| (listen_response, config_clone, actor_addr_c))
                });
                let startup_results = join_all(startup_futures).await;
                let start_child_results = startup_results.into_iter().map(|(r, c, addr)| match r {
                    Ok(Ok(socket_addr)) => {
                        let evt = ChildResult::Created(ChildCreated {
                            server_handler: ChildHandlerMinimal {
                                identity: c.identity,
                                socket_addr,
                            },
                        });
                        (Some(addr), evt)
                    }
                    Ok(Err(err)) => {
                        let evt = ChildResult::CreateErr(ChildNotCreated {
                            server_error: err,
                            identity: c.identity.clone(),
                        });
                        (None, evt)
                    }
                    Err(e) => unreachable!("{}", e),
                });

                let patch_futures = patch_jobs.into_iter().map(|(child, server_config)| {
                    child
                        .actor_address
                        .send(Patch { server_config })
                        .map(|r| (r, child))
                });
                let results = join_all(patch_futures).await;
                let patch_child_results = results.into_iter().map(|(r, child_handler)| match r {
                    Ok(Ok((route_change_set, client_config_change_set))) => {
                        let evt = ChildResult::Patched(ChildPatched {
                            server_handler: child_handler.minimal(),
                            route_change_set,
                            client_config_change_set,
                        });
                        (Some(child_handler.actor_address), evt)
                    }
                    Ok(Err(err)) => {
                        let evt = ChildResult::PatchErr(ChildNotPatched {
                            patch_error: PatchError::DidNotPatch {
                                reason: err.to_string(),
                            },
                            identity: child_handler.identity.clone(),
                        });
                        (None, evt)
                    }
                    Err(_) => unreachable!("mailbox error on patch"),
                });

                let changes = shutdown_child_results
                    .chain(start_child_results)
                    .chain(patch_child_results)
                    .collect::<Vec<_>>();

                InputChangedResponse::from_changes(changes)
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
    pub identity: ServerIdentity,
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
