use crate::system::BsSystem;
use actix::{AsyncContext, ResponseFuture};
use bsnext_dto::internal::{InitialTaskError, TaskReportAndTree};
use bsnext_input::Input;

#[derive(actix::Message)]
#[rtype(result = "Result<TaskReportAndTree, InitialTaskError>")]
pub struct ResolveInitialTasks {
    pub(crate) input: Input,
}

impl actix::Handler<ResolveInitialTasks> for BsSystem {
    type Result = ResponseFuture<Result<TaskReportAndTree, InitialTaskError>>;

    #[tracing::instrument(skip_all, name = "Handler->ResolveInitialTasks->BsSystem")]
    fn handle(&mut self, msg: ResolveInitialTasks, ctx: &mut Self::Context) -> Self::Result {
        let capabilities = self.capabilities_addr.clone();
        let (next, rx) = self.before(&msg.input, capabilities);
        ctx.notify(next);

        Box::pin(async move {
            match rx.await {
                Ok(TaskReportAndTree {
                    report,
                    tree,
                    report_map,
                }) if report.is_ok() => Ok(TaskReportAndTree {
                    report,
                    tree,
                    report_map,
                }),
                Ok(TaskReportAndTree { .. }) => Err(InitialTaskError::FailedReport),
                Err(_) => Err(InitialTaskError::FailedUnknown),
            }
        })
    }
}
