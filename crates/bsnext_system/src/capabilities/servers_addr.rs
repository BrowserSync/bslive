use crate::capabilities::Capabilities;
use actix::{Handler, Recipient, ResponseFuture};
use bsnext_core::servers_supervisor::file_changed_handler::ServersNotification;

#[derive(actix::Message)]
#[rtype(result = "Result<Recipient<ServersNotification>, anyhow::Error>")]
pub struct RequestServersAddr;

impl Handler<RequestServersAddr> for Capabilities {
    type Result = ResponseFuture<Result<Recipient<ServersNotification>, anyhow::Error>>;

    #[tracing::instrument(skip_all, name = "RequestServersAddr")]
    fn handle(&mut self, _msg: RequestServersAddr, _ctx: &mut Self::Context) -> Self::Result {
        let addr: Recipient<ServersNotification> = self.servers_addr.clone().recipient();
        Box::pin(async move { Ok(addr) })
    }
}
