use crate::server::handler_stop::Stop;
use crate::servers_supervisor::actor::ServersSupervisor;
use futures_util::future::join_all;
use std::future::Future;
use std::pin::Pin;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct StopServers;

impl actix::Handler<StopServers> for ServersSupervisor {
    type Result = Pin<Box<dyn Future<Output = ()>>>;

    fn handle(&mut self, _msg: StopServers, _ctx: &mut Self::Context) -> Self::Result {
        let aaddresses = self.handlers.clone();

        Box::pin(async move {
            tracing::debug!("stopping {} servers", aaddresses.len());
            let fts = aaddresses
                .values()
                .map(|handler| handler.actor_address.send(Stop))
                .collect::<Vec<_>>();
            join_all(fts).await;
        })
    }
}
