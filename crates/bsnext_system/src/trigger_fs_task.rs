use crate::system::BsSystem;
use crate::tasks::task_spec::TaskSpec;
use actix::{ActorFutureExt, Handler, ResponseActFuture, WrapFuture};
use bsnext_dto::internal::TaskActionStage;
use bsnext_fs::FsEventContext;
use bsnext_task::as_actor::AsActor;
use bsnext_task::invocation::{Invocation, SpecId};
use bsnext_task::task_scope::TaskScope;
use bsnext_task::task_trigger::{FsChangesTrigger, TaskTrigger, TaskTriggerSource};

#[derive(actix::Message, Debug)]
#[rtype(result = "()")]
pub struct TriggerFsTaskEvent {
    task_scope: TaskScope,
    task_trigger: FsChangesTrigger,
    task_spec: TaskSpec,
}

impl TriggerFsTaskEvent {
    pub fn new(task_scope: TaskScope, task_trigger: FsChangesTrigger, task_spec: TaskSpec) -> Self {
        Self {
            task_scope,
            task_trigger,
            task_spec,
        }
    }

    pub fn trigger(&self) -> FsChangesTrigger {
        self.task_trigger.clone()
    }

    pub fn fs_ctx(&self) -> &FsEventContext {
        &self.task_trigger.fs_event_context
    }
}

impl Handler<TriggerFsTaskEvent> for BsSystem {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: TriggerFsTaskEvent, _ctx: &mut Self::Context) -> Self::Result {
        let trigger = msg.trigger();
        let fs_ctx = msg.fs_ctx();
        let entry = self.task_spec_mapping.get(fs_ctx);
        let cloned_id = *fs_ctx;

        if let Some(entry) = entry {
            tracing::info!("ignoring concurrent task triggering: prev: {:?}", entry);
            return Box::pin(async {}.into_actor(self));
        }

        self.task_spec_mapping
            .insert(*fs_ctx, msg.task_spec.to_owned());

        let task_id = msg.task_spec.as_id();
        let spec_id = SpecId::new(task_id);

        let trigger_recipient = Box::new(msg.task_scope).into_task_recipient();
        // let comms = self.task_comms();
        let as_trigger = TaskTrigger::new(TaskTriggerSource::FsChanges(trigger));
        let invocation = Invocation::new(spec_id, as_trigger);

        Box::pin(trigger_recipient.send(invocation).into_actor(self).map(
            move |resp, actor, _ctx| {
                let runner = actor.task_spec_mapping.get(&cloned_id);
                match (resp, runner) {
                    (Ok(result), Some(task_spec)) => {
                        let (report, report_map) = result.to_report_and_map(spec_id);
                        let tree = task_spec.as_tree_with_results(&report_map);
                        actor.publish_any_event(TaskActionStage::complete(task_id, tree, report));
                    }
                    (Ok(_), _) => {
                        tracing::trace!("a triggered command completed");
                    }
                    (Err(err), _) => {
                        tracing::error!("something prevented message handling. {:?}", err);
                    }
                }
                actor.task_spec_mapping.remove(&cloned_id);
            },
        ))
    }
}
