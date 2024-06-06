use crate::server::actor::ServerActor;
use crate::server::error::ServerError;
use crate::server::handler_listen::Listen;
use crate::servers_supervisor::actor::{ChildHandler, ServersSupervisor};
use actix::{Actor, AsyncContext};
use bsnext_input::server_config::{Identity, ServerConfig};
use futures_util::future::join_all;
use futures_util::FutureExt;
use std::future::Future;

use std::pin::Pin;
use tracing::{span, Instrument, Level};

#[derive(actix::Message)]
#[rtype(result = "Vec<ChildResult>")]
pub(crate) struct StartMessage {
    pub server_configs: Vec<ServerConfig>,
}

impl actix::Handler<StartMessage> for ServersSupervisor {
    type Result = Pin<Box<dyn Future<Output = Vec<ChildResult>>>>;

    fn handle(&mut self, msg: StartMessage, ctx: &mut Self::Context) -> Self::Result {
        let span = span!(Level::TRACE, "actix::Handler<StartMessage> for Servers");
        let self_addr = ctx.address();

        Box::pin(
            async move {
                tracing::debug!("creating {} actor(s)", msg.server_configs.len());
                let fts = msg
                    .server_configs
                    .into_iter()
                    .map(|server_config| {
                        let server = ServerActor::new_from_config(server_config.clone());
                        let actor_addr = server.start();
                        let c = server_config.clone();
                        actor_addr
                            .send(Listen {
                                parent: self_addr.clone().recipient(),
                            })
                            .map(|r| (r, c))
                    })
                    .collect::<Vec<_>>();
                tracing::info!("got {} servers to listen to", fts.len());
                let results = join_all(fts).await;
                results
                    .into_iter()
                    .map(|(fut_result, server_config)| match fut_result {
                        Ok(msg_response) => match msg_response {
                            Ok((addr, actor_addr)) => {
                                tracing::debug!("✚ got listening child: {}", addr.to_string());
                                ChildResult::Ok(ChildCreated {
                                    server_handler: ChildHandler {
                                        actor_address: actor_addr,
                                        identity: server_config.identity,
                                        socket_addr: addr,
                                    },
                                })
                            }
                            Err(e) => {
                                tracing::error!("{:?}  <- {}", server_config.identity, e);
                                ChildResult::Err(ChildNotCreated {
                                    server_error: e,
                                    identity: server_config.identity.clone(),
                                })
                            }
                        },
                        Err(_e) => {
                            unreachable!("mailbox ?")
                        }
                    })
                    .collect::<Vec<ChildResult>>()
            }
            .instrument(span),
        )
    }
}

#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct ChildCreated {
    pub(crate) server_handler: ChildHandler,
}
#[derive(Debug)]
pub enum ChildResult {
    Ok(ChildCreated),
    Err(ChildNotCreated),
}

impl actix::Handler<ChildCreated> for ServersSupervisor {
    type Result = ();

    fn handle(&mut self, msg: ChildCreated, _ctx: &mut Self::Context) -> Self::Result {
        self.handlers.insert(
            msg.server_handler.identity.clone(),
            msg.server_handler.clone(),
        );
        tracing::trace!("ChildCreated child count: {}", self.handlers.len());
    }
}

#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct ChildNotCreated {
    pub server_error: ServerError,
    pub identity: Identity,
}

impl actix::Handler<ChildNotCreated> for ServersSupervisor {
    type Result = ();

    fn handle(&mut self, _msg: ChildNotCreated, _ctx: &mut Self::Context) -> Self::Result {
        // self.handlers.insert(
        //     msg.server_handler.identity.clone(),
        //     msg.server_handler.clone(),
        // );
        // tracing::trace!("ChildCreated child count: {}", self.handlers.len());
    }
}
