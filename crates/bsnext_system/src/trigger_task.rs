use crate::task::{AsActor, Task, TaskCommand};
use crate::task_list::TaskList;
use crate::BsSystem;
use actix::{ActorFutureExt, Handler, ResponseActFuture, WrapFuture};
use bsnext_dto::internal::{AnyEvent, InternalEvents, TaskReport, TaskReportAndTree};
use std::collections::HashMap;

/// A struct representing a message to trigger a specific task in the system.
/// This message will be handled by an actor in the Actix framework.
///
/// # Attributes
/// - `task`: The task that needs to be executed. This is represented by the `Task` struct,
/// which should encapsulate all the information required to identify or process the task.
/// - `cmd`: A specific command or action (`TaskCommand`) associated with the task.
/// This determines how the task should be executed or managed.
/// - `runner`: The entity (`Runner`) responsible for executing the task.
/// This can be used to specify the execution context or environment.
/// - `done`: A one-shot sender channel (`tokio::sync::oneshot::Sender<TaskReportAndTree>`)
/// to signal the completion of the task processing and provide the resulting task report and its tree structure.
///
/// # Derive Attributes
/// - `#[derive(actix::Message)]`: Indicates that this struct is a message type compatible with the Actix actor framework.
/// - `#[rtype(result = "()")]`: Specifies that the actor handling this message does not need to return any result.
#[derive(actix::Message, Debug)]
#[rtype(result = "()")]
pub struct TriggerTask {
    pub task: Task,
    pub cmd: TaskCommand,
    pub runner: TaskList,
    pub done: tokio::sync::oneshot::Sender<TaskReportAndTree>,
}

impl TriggerTask {
    pub fn new(
        task: Task,
        cmd: TaskCommand,
        runner: TaskList,
        done: tokio::sync::oneshot::Sender<TaskReportAndTree>,
    ) -> Self {
        Self {
            task,
            cmd,
            runner,
            done,
        }
    }
}

impl Handler<TriggerTask> for BsSystem {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: TriggerTask, _ctx: &mut Self::Context) -> Self::Result {
        let cmd = msg.cmd;
        let runner = msg.runner;
        let task_id = runner.as_id();
        let cmd_recipient = Box::new(msg.task).into_task_recipient();
        let done = msg.done;
        Box::pin(cmd_recipient.send(cmd).into_actor(self).map(
            move |resp, actor, _ctx| match resp {
                Ok(result) => {
                    let report = result.to_report(task_id);
                    let mut e = HashMap::new();
                    every_report(&mut e, &report);

                    let tree = runner.as_tree_with_results(&e);
                    let report_and_tree = TaskReportAndTree { report, tree };
                    actor.publish_any_event(AnyEvent::Internal(InternalEvents::TaskReport(
                        report_and_tree.clone(),
                    )));
                    match done.send(report_and_tree) {
                        Ok(_) => tracing::debug!("did finish initial"),
                        Err(_) => tracing::error!("could not send"),
                    };
                }
                _ => todo!("handle this case..."),
            },
        ))
    }
}

pub fn every_report(hm: &mut HashMap<u64, TaskReport>, report: &TaskReport) {
    hm.insert(report.id(), report.clone());
    for inner in &report.result().task_reports {
        every_report(hm, inner)
    }
}
