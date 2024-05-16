use crate::dto::{GetServersMessageResponse, ServerDTO};
use crate::servers_supervisor::actor::ServersSupervisor;

#[derive(actix::Message)]
#[rtype(result = "GetServersMessageResponse")]
pub struct GetServersMessage;

impl actix::Handler<GetServersMessage> for ServersSupervisor {
    type Result = GetServersMessageResponse;

    fn handle(&mut self, _msg: GetServersMessage, _ctx: &mut Self::Context) -> Self::Result {
        GetServersMessageResponse {
            servers: self
                .handlers
                .iter()
                .map(|(identity, child_handler)| ServerDTO {
                    identity: identity.into(),
                    socket_addr: child_handler.socket_addr.to_string(),
                })
                .collect(),
        }
    }
}
