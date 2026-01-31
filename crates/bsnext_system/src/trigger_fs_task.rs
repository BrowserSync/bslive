use crate::tasks::task_list::TaskList;
use crate::trigger_task::every_report;
use crate::BsSystem;
use actix::{ActorFutureExt, Handler, ResponseActFuture, WrapFuture};
use bsnext_dto::internal::TaskActionStage;
use bsnext_fs::FsEventContext;
use bsnext_task::as_actor::AsActor;
use bsnext_task::invocation::Invocation;
use bsnext_task::task_group::TaskGroup;
use bsnext_task::task_trigger::{TaskTrigger, TaskTriggerSource};
use std::collections::HashMap;

#[derive(actix::Message, Debug)]
#[rtype(result = "()")]
pub struct TriggerFsTaskEvent {
    task_group: TaskGroup,
    task_trigger: TaskTrigger,
    task_list: TaskList,
}

impl TriggerFsTaskEvent {
    pub fn new(task_group: TaskGroup, task_trigger: TaskTrigger, task_list: TaskList) -> Self {
        Self {
            task_group,
            task_trigger,
            task_list,
        }
    }

    pub fn cmd(&self) -> TaskTrigger {
        self.task_trigger.clone()
    }

    pub fn fs_ctx(&self) -> &FsEventContext {
        match &self.task_trigger.variant {
            TaskTriggerSource::FsChanges {
                fs_event_context, ..
            } => fs_event_context,
            TaskTriggerSource::Exec { .. } => {
                panic!("unreachable. It's a mistake to access fs_ctx here")
            }
        }
    }
}

impl Handler<TriggerFsTaskEvent> for BsSystem {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: TriggerFsTaskEvent, _ctx: &mut Self::Context) -> Self::Result {
        let trigger = msg.cmd();
        let fs_ctx = msg.fs_ctx();
        let entry = self.task_list_mapping.get(fs_ctx);
        let cloned_id = *fs_ctx;

        if let Some(entry) = entry {
            tracing::info!("ignoring concurrent task triggering: prev: {:?}", entry);
            return Box::pin(async {}.into_actor(self));
        }

        self.task_list_mapping
            .insert(*fs_ctx, msg.task_list.to_owned());
        let task_id = msg.task_list.as_id();
        let trigger_recipient = Box::new(msg.task_group).into_task_recipient();
        let one_task = Invocation(task_id, trigger);

        Box::pin(
            trigger_recipient
                .send(one_task)
                .into_actor(self)
                .map(move |resp, actor, _ctx| {
                    let runner = actor.task_list_mapping.get(&cloned_id);
                    match (resp, runner) {
                        (Ok(result), Some(runner)) => {
                            let report = result.to_report(task_id);
                            let mut e = HashMap::new();
                            every_report(&mut e, &report);

                            let tree = runner.as_tree_with_results(&e);
                            actor.publish_any_event(TaskActionStage::complete(
                                task_id, tree, report,
                            ));
                        }
                        (Ok(_), _) => {
                            tracing::trace!("a triggered command completed");
                        }
                        (Err(err), _) => {
                            tracing::error!("something prevented message handling. {:?}", err);
                        }
                    }
                    actor.task_list_mapping.remove(&cloned_id);
                }),
        )
    }
}
