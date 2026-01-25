use crate::as_actor::AsActor;
use crate::task_group::TaskGroup;
use crate::task_list::TaskList;
use crate::task_trigger::TaskTrigger;
use crate::tasks::sh_cmd::OneTask;
use crate::BsSystem;
use actix::{ActorFutureExt, Handler, ResponseActFuture, WrapFuture};
use bsnext_dto::internal::{TaskActionStage, TaskReport, TaskReportAndTree};
use std::collections::HashMap;

/// A struct representing a message to trigger a specific task in the system.
/// This message will be handled by an actor in the Actix framework.
///
/// # Derive Attributes
/// - `#[derive(actix::Message)]`: Indicates that this struct is a message type compatible with the Actix actor framework.
/// - `#[rtype(result = "()")]`: Specifies that the actor handling this message does not need to return any result.
#[derive(actix::Message, Debug)]
#[rtype(result = "()")]
pub struct TriggerTask {
    pub task_group: TaskGroup,
    pub task_list: TaskList,
    pub task_trigger: TaskTrigger,
    /// A one-shot sender channel to signal the completion of the task processing and provide the resulting task report and its tree structure.
    pub done: tokio::sync::oneshot::Sender<TaskReportAndTree>,
}

impl TriggerTask {
    pub fn new(
        task_group: TaskGroup,
        task_trigger: TaskTrigger,
        task_list: TaskList,
        done: tokio::sync::oneshot::Sender<TaskReportAndTree>,
    ) -> Self {
        Self {
            task_group,
            task_trigger,
            task_list,
            done,
        }
    }
}

impl Handler<TriggerTask> for BsSystem {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: TriggerTask, _ctx: &mut Self::Context) -> Self::Result {
        let cmd = msg.task_trigger;
        let task_list = msg.task_list;
        let task_id = task_list.as_id();
        let cmd_recipient = Box::new(msg.task_group).into_task_recipient();
        let done = msg.done;
        let comms = cmd.comms().clone();
        let tree = task_list.as_tree();
        let trigger = OneTask(task_id, cmd);
        let with_start = async move {
            let _sent = comms
                .any_event_sender
                .send(TaskActionStage::started(task_id, tree))
                .await;
            cmd_recipient.send(trigger).await
        };
        let next = with_start
            .into_actor(self)
            .map(move |resp, actor, _ctx| match resp {
                Ok(result) => {
                    let report = result.to_report(task_id);
                    let mut e = HashMap::new();
                    every_report(&mut e, &report);

                    let tree = task_list.as_tree_with_results(&e);
                    let report_and_tree = TaskReportAndTree {
                        report: report.clone(),
                        tree: tree.clone(),
                    };
                    actor.publish_any_event(TaskActionStage::complete(task_id, tree, report));
                    match done.send(report_and_tree) {
                        Ok(_) => tracing::debug!("did finish initial"),
                        Err(_) => tracing::error!("could not send"),
                    };
                }
                _ => todo!("handle this case..."),
            });
        Box::pin(next)
    }
}

pub fn every_report(hm: &mut HashMap<u64, TaskReport>, report: &TaskReport) {
    hm.insert(report.id(), report.clone());
    for inner in &report.result().task_reports {
        every_report(hm, inner)
    }
}
