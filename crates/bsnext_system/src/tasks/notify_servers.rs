use actix::{Actor, Handler, ResponseFuture};
use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;
use bsnext_dto::internal::{InvocationId, TaskResult};
use bsnext_task::invocation::Invocation;
use bsnext_task::task_trigger::TaskTriggerSource;

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

impl Handler<Invocation> for NotifyServers {
    type Result = ResponseFuture<TaskResult>;

    fn handle(
        &mut self,
        Invocation(_id, trigger): Invocation,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        tracing::debug!("NotifyServers::TaskCommand");
        let comms = trigger.comms();
        todo!("NotifyServers::TaskCommand");
        // let Some(sender) = comms.servers_recip.clone() else {
        //     todo!("cannot get here?")
        // };
        // match trigger.variant {
        //     TaskTriggerSource::FsChanges {
        //         changes,
        //         fs_event_context,
        //         ..
        //     } => sender.do_send(FilesChanged {
        //         paths: changes.clone(),
        //         ctx: fs_event_context,
        //     }),
        //     TaskTriggerSource::Exec { .. } => {
        //         todo!("I cannot accept this")
        //     }
        // }
        Box::pin(async { TaskResult::ok(InvocationId(0)) })
    }
}
