use crate::server::handler_change::Change;
use crate::servers_supervisor::actor::ServersSupervisor;
use std::path::PathBuf;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct FileChanged {
    pub path: PathBuf,
    pub id: u64,
}

impl actix::Handler<FileChanged> for ServersSupervisor {
    type Result = ();

    fn handle(&mut self, msg: FileChanged, _ctx: &mut Self::Context) -> Self::Result {
        for child in self.handlers.values() {
            if child.identity.as_id() == msg.id {
                child.actor_address.do_send(Change::fs(&msg.path))
            }
        }
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct FilesChanged {
    pub paths: Vec<PathBuf>,
    pub id: u64,
}

impl actix::Handler<FilesChanged> for ServersSupervisor {
    type Result = ();

    fn handle(&mut self, msg: FilesChanged, _ctx: &mut Self::Context) -> Self::Result {
        for child in self.handlers.values() {
            if child.identity.as_id() == msg.id {
                child.actor_address.do_send(Change::fs_many(&msg.paths))
            }
        }
    }
}
