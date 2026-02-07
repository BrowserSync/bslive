use actix::{Actor, Addr, Handler, ResponseFuture};
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;
use bsnext_dto::internal::{InvocationId, TaskResult};
use bsnext_task::invocation::Invocation;
use bsnext_task::task_trigger::TaskTriggerSource;

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
        Invocation { trigger, .. }: Invocation,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        tracing::debug!("NotifyServers::TaskCommand");
        let addr = self.addr.clone();
        match trigger.variant {
            TaskTriggerSource::FsChanges {
                changes,
                fs_event_context,
                ..
            } => addr.do_send(FilesChanged {
                paths: changes.clone(),
                ctx: fs_event_context,
            }),
            TaskTriggerSource::Exec { .. } => {
                todo!("I cannot accept this")
            }
        }
        Box::pin(async { TaskResult::ok(InvocationId(0)) })
    }
}

pub struct NotifyServersNoOp;
impl Actor for NotifyServersNoOp {
    type Context = actix::Context<Self>;
}
impl Handler<Invocation> for NotifyServersNoOp {
    type Result = ResponseFuture<TaskResult>;
    fn handle(&mut self, _invocation: Invocation, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(async { TaskResult::ok(InvocationId(0)) })
    }
}
