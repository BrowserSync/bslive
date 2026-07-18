use crate::system::BsSystem;
use actix::ResponseFuture;
use bsnext_core::servers_supervisor::get_servers_handler::GetActiveServers;
use bsnext_dto::internal::ServerError;
use bsnext_dto::GetActiveServersResponse;
#[derive(actix::Message)]
#[rtype(result = "Result<GetActiveServersResponse, ServerError>")]
pub struct ReadActiveServers;

impl actix::Handler<ReadActiveServers> for BsSystem {
    type Result = ResponseFuture<Result<GetActiveServersResponse, ServerError>>;

    fn handle(&mut self, _msg: ReadActiveServers, _ctx: &mut Self::Context) -> Self::Result {
        let cloned_address = self.servers().clone();

        Box::pin(async move {
            match cloned_address.send(GetActiveServers).await {
                Ok(resp) => Ok(resp),
                Err(e) => Err(ServerError::Unknown(e.to_string())),
            }
        })
    }
}
