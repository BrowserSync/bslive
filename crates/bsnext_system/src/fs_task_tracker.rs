use crate::invoke_scope::InvokeScope;
use crate::tasks::task_spec::TaskSpec;
use actix::ActorFutureExt;
use actix::{Actor, Handler, Recipient, ResponseActFuture, WrapFuture};
use bsnext_dto::internal::TaskReportAndTree;
use bsnext_fs::FsEventContext;
use bsnext_task::task_trigger::{FsChangesTrigger, TaskTrigger, TaskTriggerSource};
use std::collections::HashMap;
use tokio::sync::oneshot::Receiver;
use tracing::Level;

#[derive(Debug)]
pub struct FsTaskTracker {
    task_spec_mapping: HashMap<FsEventContext, TaskSpec>,
    spec_invoker: Recipient<InvokeScope>,
}

impl Actor for FsTaskTracker {
    type Context = actix::Context<Self>;
}

impl FsTaskTracker {
    pub fn new(invoker: Recipient<InvokeScope>) -> Self {
        Self {
            task_spec_mapping: Default::default(),
            spec_invoker: invoker,
        }
    }
}

impl Handler<TriggerFsTask> for FsTaskTracker {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: TriggerFsTask, _ctx: &mut Self::Context) -> Self::Result {
        let span = tracing::debug_span!("TriggerFsTask");
        let _g = span.entered();
        let trigger = msg.trigger();
        debug_trigger(&trigger);

        let fs_ctx = trigger.fs_ctx().to_owned();
        let task_spec = msg.task_spec;
        debug_spec(&task_spec);

        let entry = self.task_spec_mapping.get(&fs_ctx);
        let cloned_id = fs_ctx;

        if let Some(entry) = entry {
            tracing::info!("ignoring concurrent task triggering: prev: {:?}", entry);
            return Box::pin(async {}.into_actor(self));
        }

        self.task_spec_mapping.insert(fs_ctx, task_spec.to_owned());

        let task_trigger = TaskTrigger::new(TaskTriggerSource::FsChanges(trigger));
        let (tx, rx) = tokio::sync::oneshot::channel::<TaskReportAndTree>();
        let invoke_spec = InvokeScope::new(task_trigger, task_spec, tx);
        self.spec_invoker.do_send(invoke_spec);

        Box::pin(run(rx).into_actor(self).map(move |_resp, actor, _ctx| {
            actor.task_spec_mapping.remove(&cloned_id);
        }))
    }
}

fn debug_spec(task_spec: &TaskSpec) {
    if tracing::enabled!(Level::DEBUG) {
        let tree = task_spec.as_tree();
        tracing::debug!("task spec to execute:\n{tree}");
    }
}

fn debug_trigger(trigger: &FsChangesTrigger) {
    if tracing::enabled!(Level::DEBUG) {
        tracing::debug!(
            "received {} changes. (further details in trace)",
            trigger.changes().len()
        );
    }
    if tracing::enabled!(Level::TRACE) {
        for pb in trigger.changes() {
            tracing::trace!(?pb);
        }
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

/// Message to trigger execution of a filesystem-watched task.
///
/// This provides a simple binary concurrency guard: if a task is already running,
/// new triggers are ignored until the current run completes. This prevents
/// rapid file changes from spawning overlapping task executions.
///
/// Unlike permit-based semaphores, this is an on/off switch - either the task
/// is running or it isn't. The guard is tracked per-task via `task_spec_mapping`.
///
/// Execution still delegates to `InvokeScope` like other task types, but with
/// this gating layer in front. Not needed for CI tasks where runs are naturally serialized.
#[derive(actix::Message, Debug)]
#[rtype(result = "()")]
pub struct TriggerFsTask {
    task_spec: TaskSpec,
    task_trigger: FsChangesTrigger,
}

impl TriggerFsTask {
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
