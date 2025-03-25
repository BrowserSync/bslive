use crate::task::TaskCommand;
use actix::{Actor, Addr, Handler, ResponseFuture};
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;
use bsnext_fs::FsEventContext;

pub struct NotifyServers {
    addr: Addr<ServersSupervisor>,
}

impl NotifyServers {
    pub fn new(addr: Addr<ServersSupervisor>) -> Self {
        Self { addr }
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
        tracing::debug!("NotifyServers::TaskCommand");
        match msg {
            TaskCommand::Changes {
                changes,
                fs_event_context,
            } => self.addr.do_send(FilesChanged {
                paths: changes,
                ctx: fs_event_context,
            }),
        }
        Box::pin(async {})
    }
}
