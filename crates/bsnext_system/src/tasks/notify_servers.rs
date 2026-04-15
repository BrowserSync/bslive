use crate::capabilities::servers_addr::RequestServersAddr;
use actix::{Actor, Handler, Recipient, ResponseFuture};
use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;
use bsnext_task::invocation::Invocation;
use bsnext_task::invocation_result::InvocationResult;
use bsnext_task::task_trigger::{FsChangesTrigger, TaskTriggerSource};

pub struct NotifyServersReady {
    addr: Recipient<RequestServersAddr>,
}

impl NotifyServersReady {
    pub fn new(addr: Recipient<RequestServersAddr>) -> Self {
        Self { addr }
    }
}

impl Actor for NotifyServersReady {
    type Context = actix::Context<Self>;
}

impl Handler<Invocation> for NotifyServersReady {
    type Result = ResponseFuture<InvocationResult>;

    fn handle(&mut self, invocation: Invocation, _ctx: &mut Self::Context) -> Self::Result {
        tracing::debug!("NotifyServersReady::TaskCommand");
        let addr = self.addr.clone();
        let spec_id = invocation.path().to_owned();
        let source = invocation.trigger().source();
        Box::pin({
            let source = source.clone();
            async move {
                let f = match source {
                    TaskTriggerSource::FsChanges(trigger) => do_it(addr, &trigger).await,
                    TaskTriggerSource::Exec(..) => {
                        todo!("I cannot accept this")
                    }
                };
                match f {
                    Ok(_) => InvocationResult::ok(spec_id),
                    Err(_) => InvocationResult::err_message(spec_id, "couldn't notify servers"),
                }
            }
        })
    }
}

async fn do_it(
    addr: Recipient<RequestServersAddr>,
    trigger: &FsChangesTrigger,
) -> anyhow::Result<()> {
    let next = addr.send(RequestServersAddr).await??;
    next.do_send(FilesChanged {
        paths: trigger.changes().to_owned(),
        ctx: trigger.fs_ctx().to_owned(),
    });
    Ok(())
}
