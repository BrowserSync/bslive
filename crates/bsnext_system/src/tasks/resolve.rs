use crate::BsSystem;
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
        let Some(addr) = self.capabilities_addr.as_ref() else {
            todo!("unreachable")
        };
        let (next, rx) = self.before(&msg.input, addr.clone());
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
