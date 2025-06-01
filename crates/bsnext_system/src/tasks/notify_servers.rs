use crate::task::TaskCommand;
use actix::{Actor, Handler, ResponseFuture};
use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;
use bsnext_dto::internal::{InvocationId, TaskResult};

#[derive(Default)]
pub struct NotifyServers {}

impl NotifyServers {
    pub fn new() -> Self {
        Self {}
    }
}

impl Actor for NotifyServers {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::debug!(actor.lifecycle = "started", "NotifyServers");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::debug!(actor.lifecycle = "stopped", "NotifyServers");
    }
}

impl Handler<TaskCommand> for NotifyServers {
    type Result = ResponseFuture<TaskResult>;

    fn handle(&mut self, msg: TaskCommand, _ctx: &mut Self::Context) -> Self::Result {
        tracing::debug!("NotifyServers::TaskCommand");
        let comms = msg.comms();
        let Some(sender) = comms.servers_recip.clone() else {
            todo!("cannot get here?")
        };
        match msg {
            TaskCommand::Changes {
                changes,
                fs_event_context,
                ..
            } => sender.do_send(FilesChanged {
                paths: changes.clone(),
                ctx: fs_event_context.clone(),
            }),
        }
        Box::pin(async { TaskResult::ok(InvocationId(0)) })
    }
}
