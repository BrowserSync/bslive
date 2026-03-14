use crate::capabilities::Capabilities;
use actix::{Handler, Recipient, ResponseFuture};
use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;

#[derive(actix::Message)]
#[rtype(result = "Result<Recipient<FilesChanged>, anyhow::Error>")]
pub struct RequestServersAddr;

impl Handler<RequestServersAddr> for Capabilities {
    type Result = ResponseFuture<Result<Recipient<FilesChanged>, anyhow::Error>>;

    #[tracing::instrument(skip_all, name = "RequestServersAddr")]
    fn handle(&mut self, _msg: RequestServersAddr, _ctx: &mut Self::Context) -> Self::Result {
        let addr: Recipient<FilesChanged> = self.servers_addr.clone().recipient();
        Box::pin(async move { Ok(addr) })
    }
}
