use crate::server::handler_change::{Change, ChangeWithSpan};
use crate::servers_supervisor::actor::ServersSupervisor;
use bsnext_fs::FsEventContext;
use std::path::PathBuf;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct FileChanged {
    pub path: PathBuf,
    pub ctx: FsEventContext,
}

impl actix::Handler<FileChanged> for ServersSupervisor {
    type Result = ();

    fn handle(&mut self, msg: FileChanged, _ctx: &mut Self::Context) -> Self::Result {
        for child in self.handlers.values() {
            if child.identity.as_id() == msg.ctx.id() {
                let outgoing = ChangeWithSpan::new(Change::fs(&msg.path));
                child.actor_address.do_send(outgoing)
            }
        }
    }
}

#[derive(Debug, Clone, actix::Message)]
#[rtype(result = "()")]
pub struct FilesChanged {
    pub paths: Vec<PathBuf>,
    pub ctx: FsEventContext,
}

impl actix::Handler<FilesChanged> for ServersSupervisor {
    type Result = ();

    fn handle(&mut self, msg: FilesChanged, _ctx: &mut Self::Context) -> Self::Result {
        tracing::debug!("looking at {} handlers", self.handlers.len());
        tracing::debug!(ctx = ?msg.ctx, "msg ctx");

        // special-case when there's only 1 server.
        if self.handlers.len() == 1 {
            tracing::debug!("skipping server identity checks because there's only 1 server");
            let Some((_id, handler)) = self.handlers.iter().nth(0) else {
                todo!("impossible")
            };
            let outgoing = ChangeWithSpan::new(Change::fs_many(&msg.paths));
            handler.actor_address.do_send(outgoing);
        } else {
            for child in self.handlers.values() {
                if child.identity.as_id() == msg.ctx.id() {
                    let outgoing = ChangeWithSpan::new(Change::fs_many(&msg.paths));
                    child.actor_address.do_send(outgoing);
                } else {
                    tracing::debug!("child identity didn't match msg.ctx.id");
                    tracing::debug!("  -   child: {}", child.identity.as_id());
                    tracing::debug!("  - msg.ctx: {}", msg.ctx.id());
                }
            }
        }
    }
}
