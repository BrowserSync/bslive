use crate::servers_supervisor::actor::ServersSupervisor;
use crate::servers_supervisor::file_changed_handler::FilesChanged;
use actix::AsyncContext;
use bsnext_dto::{GetServersMessageResponse, ServerDTO};

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
                    id: identity.as_id().to_string(),
                    identity: identity.into(),
                    socket_addr: child_handler.socket_addr.to_string(),
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, actix::Message)]
#[rtype(result = "()")]
pub enum IncomingEvents {
    FilesChanged(FilesChanged),
}

impl actix::Handler<IncomingEvents> for ServersSupervisor {
    type Result = ();

    fn handle(&mut self, msg: IncomingEvents, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            IncomingEvents::FilesChanged(files_changed) => ctx.notify(files_changed),
        }
    }
}
