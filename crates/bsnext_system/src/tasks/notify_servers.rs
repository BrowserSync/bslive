use crate::task::TaskCommand;
use actix::{Actor, Addr, Handler, ResponseFuture};
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;
use bsnext_fs::FsEventContext;

pub struct NotifyServers {
    addr: Addr<ServersSupervisor>,
    ctx: FsEventContext,
}

impl NotifyServers {
    pub fn new(addr: &Addr<ServersSupervisor>, ctx: FsEventContext) -> Self {
        Self {
            addr: addr.clone(),
            ctx,
        }
    }
}

impl Actor for NotifyServers {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        tracing::debug!("started");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        tracing::debug!("stopped");
    }
}

impl Handler<TaskCommand> for NotifyServers {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: TaskCommand, ctx: &mut Self::Context) -> Self::Result {
        match msg {
            TaskCommand::Changes(paths) => self.addr.do_send(FilesChanged {
                paths,
                ctx: self.ctx.clone(),
            }),
        }
        Box::pin(async {})
    }
}

pub struct Chain {}
