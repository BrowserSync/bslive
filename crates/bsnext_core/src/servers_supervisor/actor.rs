use crate::server::handler_stop::Stop;
use actix::{Actor, Addr, Running};

use crate::server::actor::ServerActor;
use crate::servers_supervisor::start_handler::{ChildResult, StartMessage};

use crate::server::handler_patch::Patch;
use bsnext_input::server_config::Identity;
use bsnext_input::Input;
use std::collections::HashSet;
use std::future::Future;
use std::net::SocketAddr;

use bsnext_dto::{ServerChange, ServerChangeSet, ServerChangeSetItem};
use std::pin::Pin;
use tokio::sync::oneshot::Sender;
use tracing::{span, Instrument, Level};

#[derive(Debug)]
pub struct ServersSupervisor {
    pub(crate) handlers: std::collections::HashMap<Identity, ChildHandler>,
    tx: Option<Sender<()>>,
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
    ) -> Pin<Box<impl Future<Output = ServerChangeSet> + Sized>> {
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
                let mut changeset = ServerChangeSet { items: vec![] };
                for x in shutdown_jobs {
                    tracing::debug!("stopping {:?}", x.identity);
                    changeset.items.push(ServerChangeSetItem {
                        identity: (&x.identity).into(),
                        change: ServerChange::Stopped {
                            bind_address: x.socket_addr.to_string(),
                        },
                    });
                    match x.actor_address.send(Stop).await {
                        Ok(_) => {
                            self_addr.do_send(ChildStopped {
                                identity: x.identity.clone(),
                            });
                        }
                        Err(_) => {
                            tracing::error!("couldn't send Stop2 {:?}", x.identity);
                        }
                    }
                }

                if !start_jobs.is_empty() {
                    tracing::debug!("starting {:?} servers", start_jobs.len());
                    match self_addr
                        .send(StartMessage {
                            server_configs: start_jobs,
                        })
                        .await
                    {
                        Ok(output) => {
                            for x in output {
                                match x {
                                    ChildResult::Ok(child_created) => {
                                        tracing::info!("child_created");
                                        let iden = child_created.server_handler.identity.clone();
                                        self_addr.do_send(child_created);
                                        changeset.items.push(ServerChangeSetItem {
                                            identity: (&iden).into(),
                                            change: ServerChange::Started,
                                        })
                                    }
                                    ChildResult::Err(e) => {
                                        tracing::info!(?e, "child not created");
                                        changeset.items.push(ServerChangeSetItem {
                                            identity: (&e.identity).into(),
                                            change: ServerChange::Errored {
                                                error: format!("{:?}", e.server_error),
                                            },
                                        })
                                    }
                                }
                            }
                        }
                        Err(_) => tracing::error!("could not send StartMessage to self"),
                    };
                }

                for (child, config) in patch_jobs {
                    tracing::debug!("patching {:?}", child.identity);
                    child.actor_address.do_send(Patch {
                        server_config: config,
                    });
                    changeset.items.push(ServerChangeSetItem {
                        identity: (&child.identity).into(),
                        change: ServerChange::Patched,
                    })
                }

                changeset
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

#[derive(Debug, Clone)]
pub struct ChildHandler {
    pub actor_address: Addr<ServerActor>,
    pub identity: Identity,
    pub socket_addr: SocketAddr,
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
