use crate::system::BsSystem;
use crate::tasks::task_spec::TaskSpec;
use actix::ResponseFuture;
use bsnext_dto::internal::InitialTaskError;
use bsnext_input::Input;

#[derive(actix::Message)]
#[rtype(result = "Result<TaskSpec, InitialTaskError>")]
pub struct ResolveInitialTasks {
    input: Input,
}

impl ResolveInitialTasks {
    pub fn new(input: Input) -> Self {
        Self { input }
    }
}

impl actix::Handler<ResolveInitialTasks> for BsSystem {
    type Result = ResponseFuture<Result<TaskSpec, InitialTaskError>>;

    // Note: This handler is implemented as an async call to facilitate future upgrades
    // when task resolution logic becomes more complex and requires asynchronous operations.
    #[tracing::instrument(skip_all, name = "ResolveInitialTasks")]
    fn handle(&mut self, msg: ResolveInitialTasks, _ctx: &mut Self::Context) -> Self::Result {
        let task_spec = self.before(&msg.input);
        Box::pin(async move { Ok(task_spec) })
    }
}
