use crate::servers_supervisor::actor::ServersSupervisor;
use crate::servers_supervisor::file_changed_handler::FilesChanged;
use actix::AsyncContext;
use bsnext_dto::{ActiveServer, GetActiveServersResponse};

#[derive(actix::Message)]
#[rtype(result = "GetActiveServersResponse")]
pub struct GetActiveServers;

impl actix::Handler<GetActiveServers> for ServersSupervisor {
    type Result = GetActiveServersResponse;

    fn handle(&mut self, _msg: GetActiveServers, _ctx: &mut Self::Context) -> Self::Result {
        GetActiveServersResponse {
            servers: self
                .handlers
                .iter()
                .map(|(identity, child_handler)| ActiveServer {
                    identity: identity.clone(),
                    socket_addr: child_handler.socket_addr,
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
