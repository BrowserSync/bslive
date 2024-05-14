use crate::server::actor::ServerActor;
use actix::ActorContext;
use std::future::Future;
use std::pin::Pin;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Stop;

impl actix::Handler<Stop> for ServerActor {
    type Result = Pin<Box<dyn Future<Output = ()>>>;

    fn handle(&mut self, _msg: Stop, ctx: &mut Self::Context) -> Self::Result {
        tracing::trace!("{:?} [ServerActor::Stop2]", self.addr);

        // don't accept any more messages
        let Some(signals) = self.signals.take() else {
            todo!("should be unreachable. close signal can only be sent once")
        };
        let Some(handle) = signals.axum_server_handle else {
            todo!("cannot get here handle cannnot be absent can it??")
        };
        tracing::trace!("{:?} using handle to shutdown", self.config.identity);
        handle.shutdown();
        ctx.stop();
        if let Some(complete_msg_receiver) = signals.complete_mdg_receiver {
            tracing::debug!("{:?} confirmed closed via signal", self.addr);
            Box::pin(async move {
                match complete_msg_receiver.await {
                    Ok(_) => {}
                    Err(e) => tracing::error!("failed to get complete message {e}"),
                }
            })
        } else {
            todo!("cannot get here?")
        }
    }
}
