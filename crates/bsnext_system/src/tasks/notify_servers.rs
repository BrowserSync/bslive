use crate::task::TaskCommand;
use actix::{Actor, Handler, ResponseFuture};
use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;

pub struct NotifyServers {}

impl NotifyServers {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for NotifyServers {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        tracing::debug!("started");
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        tracing::debug!(" x stopped NotifyServers")
    }
}

impl Handler<TaskCommand> for NotifyServers {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: TaskCommand, ctx: &mut Self::Context) -> Self::Result {
        tracing::debug!("NotifyServers::TaskCommand");
        let comms = msg.comms();
        let sender = comms.servers_addr.clone();
        match msg {
            TaskCommand::Changes {
                changes,
                fs_event_context,
                ..
            } => sender.do_send(FilesChanged {
                paths: changes,
                ctx: fs_event_context,
            }),
        }
        Box::pin(async {})
    }
}
