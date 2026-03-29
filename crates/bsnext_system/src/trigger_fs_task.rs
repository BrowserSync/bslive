use crate::invoke_scope::InvokeScope;
use crate::system::BsSystem;
use crate::tasks::task_spec::TaskSpec;
use actix::{ActorFutureExt, AsyncContext, Handler, ResponseActFuture, WrapFuture};
use bsnext_dto::internal::{InitialTaskError, TaskActionStage, TaskReportAndTree};
use bsnext_task::as_actor::AsActor;
use bsnext_task::invocation::Invocation;
use bsnext_task::task_trigger::{FsChangesTrigger, TaskTrigger, TaskTriggerSource};
use bsnext_task::ContentId;
use std::convert::Infallible;
use tokio::sync::oneshot::Receiver;

#[derive(actix::Message, Debug)]
#[rtype(result = "()")]
pub struct TriggerFsTaskEvent {
    task_spec: TaskSpec,
    task_trigger: FsChangesTrigger,
}

impl TriggerFsTaskEvent {
    pub fn new(task_spec: TaskSpec, task_trigger: FsChangesTrigger) -> Self {
        Self {
            task_spec,
            task_trigger,
        }
    }

    pub fn trigger(&self) -> FsChangesTrigger {
        self.task_trigger.clone()
    }
}

impl Handler<TriggerFsTaskEvent> for BsSystem {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: TriggerFsTaskEvent, ctx: &mut Self::Context) -> Self::Result {
        let trigger = msg.trigger();
        let fs_ctx = trigger.fs_ctx().to_owned();
        let task_spec = msg.task_spec;
        let comms = self.task_comms();

        let entry = self.task_spec_mapping.get(&fs_ctx);
        let cloned_id = fs_ctx;

        if let Some(entry) = entry {
            tracing::info!("ignoring concurrent task triggering: prev: {:?}", entry);
            return Box::pin(async {}.into_actor(self));
        }

        let task_scope = task_spec
            .clone()
            .to_task_scope(self.servers().clone(), self.capabilities().clone());

        self.task_spec_mapping.insert(fs_ctx, task_spec.to_owned());

        let task_trigger = TaskTrigger::new(TaskTriggerSource::FsChanges(trigger));
        let (tx, rx) = tokio::sync::oneshot::channel::<TaskReportAndTree>();
        let invoke_scope = InvokeScope::new(task_scope, task_trigger, task_spec, comms, tx);
        ctx.notify(invoke_scope);

        let top_level_scope = run(rx);
        Box::pin(
            top_level_scope
                .into_actor(self)
                .map(move |resp, actor, _ctx| {
                    actor.task_spec_mapping.remove(&cloned_id);
                }),
        )
    }
}

async fn run(rx: Receiver<TaskReportAndTree>) -> anyhow::Result<()> {
    let output = rx.await?;
    if !output.report.is_ok() {
        tracing::debug!("✅TriggerFsTaskEvent triggered a invoke scope and succeeded")
    } else {
        tracing::debug!("❌ TriggerFsTaskEvent triggered a invoke scope and failed")
    }
    Ok(())
}
