use crate::capabilities::servers_addr::RequestServersAddr;
use actix::{Actor, Handler, Recipient, ResponseFuture};
use bsnext_core::servers_supervisor::file_changed_handler::ServersNotification;
use bsnext_input::bs_live_built_in_task::ClientNotification;
use bsnext_task::invocation::Invocation;
use bsnext_task::invocation_result::InvocationResult;
use bsnext_task::task_trigger::{TaskTrigger, TaskTriggerSource};

pub struct NotifyClientsReady {
    addr: Recipient<RequestServersAddr>,
    notification: ClientNotification,
}

impl NotifyClientsReady {
    pub fn new(addr: Recipient<RequestServersAddr>, notification: ClientNotification) -> Self {
        Self { addr, notification }
    }
}

impl Actor for NotifyClientsReady {
    type Context = actix::Context<Self>;
}

impl Handler<Invocation> for NotifyClientsReady {
    type Result = ResponseFuture<InvocationResult>;

    fn handle(&mut self, invocation: Invocation, _ctx: &mut Self::Context) -> Self::Result {
        tracing::debug!("NotifyClientsReady::TaskCommand");
        let addr = self.addr.clone();
        let spec_id = invocation.path().to_owned();
        Box::pin({
            let mut source = self.notification.to_owned();
            let ClientNotification::DisplayMessage(ref mut dm) = source;
            let trigger = invocation.trigger();
            dm.reason = Some(trigger_str(trigger));
            async move {
                let f = do_it(addr, &source).await;
                match f {
                    Ok(_) => InvocationResult::ok(spec_id),
                    Err(_) => InvocationResult::err_message(spec_id, "couldn't notify servers"),
                }
            }
        })
    }
}

fn trigger_str(trigger: &TaskTrigger) -> String {
    match trigger.source() {
        TaskTriggerSource::FsChanges(fs) => fs
            .changes()
            .iter()
            .map(|pb| format!("{}", pb.display()))
            .collect::<Vec<_>>()
            .join(","),
        TaskTriggerSource::Exec(_) => "exec".to_string(),
    }
}

async fn do_it(
    addr: Recipient<RequestServersAddr>,
    notification: &ClientNotification,
) -> anyhow::Result<()> {
    let next = addr.send(RequestServersAddr).await??;
    next.do_send(ServersNotification::ClientNotification(
        notification.to_owned(),
    ));
    Ok(())
}
