use crate::task::{AsActor, Task, TaskCommand};
use crate::task_list::TaskList;
use crate::trigger_task::every_report;
use crate::BsSystem;
use actix::{ActorFutureExt, Handler, ResponseActFuture, WrapFuture};
use bsnext_dto::internal::{AnyEvent, InternalEvents, TaskReportAndTree};
use bsnext_fs::FsEventContext;
use std::collections::HashMap;

#[derive(actix::Message, Debug)]
#[rtype(result = "()")]
pub struct TriggerFsTask {
    task: Task,
    cmd: TaskCommand,
    runner: TaskList,
}

impl TriggerFsTask {
    pub fn new(task: Task, cmd: TaskCommand, runner: TaskList) -> Self {
        Self { task, cmd, runner }
    }

    pub fn cmd(&self) -> TaskCommand {
        self.cmd.clone()
    }

    pub fn fs_ctx(&self) -> &FsEventContext {
        match &self.cmd {
            TaskCommand::Changes {
                fs_event_context, ..
            } => fs_event_context,
            TaskCommand::Exec { .. } => {
                panic!("unreachable. It's a mistake to access fs_ctx here")
            }
        }
    }
}

impl Handler<TriggerFsTask> for BsSystem {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: TriggerFsTask, _ctx: &mut Self::Context) -> Self::Result {
        let cmd = msg.cmd();
        let fs_ctx = msg.fs_ctx();
        let entry = self.tasks.get(fs_ctx);
        let cloned_id = *fs_ctx;

        if let Some(entry) = entry {
            tracing::info!("ignoring concurrent task triggering: prev: {:?}", entry);
            return Box::pin(async {}.into_actor(self));
        }

        self.tasks.insert(*fs_ctx, msg.runner.to_owned());
        let task_id = msg.runner.as_id();
        let cmd_recipient = Box::new(msg.task).into_task_recipient();

        Box::pin(
            cmd_recipient
                .send(cmd)
                .into_actor(self)
                .map(move |resp, actor, _ctx| {
                    let runner = actor.tasks.get(&cloned_id);
                    match (resp, runner) {
                        (Ok(result), Some(runner)) => {
                            let report = result.to_report(task_id);
                            let mut e = HashMap::new();
                            every_report(&mut e, &report);

                            let tree = runner.as_tree_with_results(&e);
                            actor.publish_any_event(AnyEvent::Internal(
                                InternalEvents::TaskReport(TaskReportAndTree { report, tree }),
                            ));
                        }
                        (Ok(_), _) => {
                            tracing::trace!("a triggered command completed");
                        }
                        (Err(err), _) => {
                            tracing::error!("something prevented message handling. {:?}", err);
                        }
                    }
                    actor.tasks.remove(&cloned_id);
                }),
        )
    }
}
