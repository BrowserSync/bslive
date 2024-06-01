use crate::server::handler_change::{Change, ChangeWithSpan};
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

    #[tracing::instrument(skip_all, name = "FileChanged for ServersSupervisor")]
    fn handle(&mut self, msg: FileChanged, _ctx: &mut Self::Context) -> Self::Result {
        for child in self.handlers.values() {
            if child.identity.as_id() == msg.id {
                let outgoing = ChangeWithSpan {
                    evt: Change::fs(&msg.path),
                };
                child.actor_address.do_send(outgoing)
            }
        }
    }
}

#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct FilesChanged {
    pub paths: Vec<PathBuf>,
    pub id: u64,
}

impl actix::Handler<FilesChanged> for ServersSupervisor {
    type Result = ();

    #[tracing::instrument(skip_all, name = "FilesChanged for ServersSupervisor")]
    fn handle(&mut self, msg: FilesChanged, _ctx: &mut Self::Context) -> Self::Result {
        tracing::debug!("sending message to {} handlers", self.handlers.len());
        for child in self.handlers.values() {
            let outgoing = ChangeWithSpan {
                evt: Change::fs_many(&msg.paths),
            };
            child.actor_address.do_send(outgoing);
        }
    }
}
