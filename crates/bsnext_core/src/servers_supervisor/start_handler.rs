use crate::server::actor::ServerActor;
use crate::server::handler_listen::Listen;
use crate::servers_supervisor::actor::{ChildHandler, ServersSupervisor};
use actix::{Actor, AsyncContext};
use bsnext_input::server_config::ServerConfig;
use futures_util::future::join_all;
use futures_util::FutureExt;
use std::future::Future;
use std::pin::Pin;
use tracing::{span, Instrument, Level};

#[derive(actix::Message)]
#[rtype(result = "()")]
pub(crate) struct StartMessage {
    pub server_configs: Vec<ServerConfig>,
}

impl actix::Handler<StartMessage> for ServersSupervisor {
    type Result = Pin<Box<dyn Future<Output = ()>>>;

    fn handle(&mut self, msg: StartMessage, ctx: &mut Self::Context) -> Self::Result {
        let span = span!(Level::TRACE, "actix::Handler<StartMessage> for Servers");
        let self_addr = ctx.address();

        let workload = async move {
            tracing::debug!("creating {} actor(s)", msg.server_configs.len());
            let fts = msg
                .server_configs
                .into_iter()
                .map(|server_config| {
                    let server = ServerActor::new_from_config(server_config.clone());
                    let actor_addr = server.start();
                    let c = server_config.clone();
                    actor_addr.send(Listen).map(|r| (r, c))
                })
                .collect::<Vec<_>>();

            let results = join_all(fts).await;
            for (fut_result, server_config) in results {
                match fut_result {
                    Ok(msg_response) => match msg_response {
                        Ok((addr, actor_addr)) => {
                            tracing::debug!("✚ got listening child: {}", addr.to_string());
                            self_addr.do_send(ChildCreated {
                                server_handler: ChildHandler {
                                    actor_address: actor_addr,
                                    identity: server_config.identity,
                                    socket_addr: addr,
                                },
                            });
                        }
                        Err(e) => {
                            tracing::error!("{:?}  <- {}", server_config.identity, e)
                        }
                    },
                    Err(e) => tracing::error!("  <- [m] {}", e),
                }
            }
        }
        .instrument(span);

        Box::pin(workload)
    }
}

#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct ChildCreated {
    server_handler: ChildHandler,
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
