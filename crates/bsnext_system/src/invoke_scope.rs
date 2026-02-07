use crate::tasks::task_spec::TaskSpec;
use crate::BsSystem;
use actix::{ActorFutureExt, Handler, ResponseActFuture, StreamHandler, WrapFuture};
use bsnext_dto::internal::{TaskActionStage, TaskReport, TaskReportAndTree};
use bsnext_task::as_actor::AsActor;
use bsnext_task::invocation::Invocation;
use bsnext_task::task_scope::TaskScope;
use bsnext_task::task_trigger::{TaskComms, TaskTrigger};
use std::collections::HashMap;
use tokio_stream::wrappers::ReceiverStream;

/// A struct representing a message to trigger a specific task in the system.
/// This message will be handled by an actor in the Actix framework.
///
/// # Derive Attributes
/// - `#[derive(actix::Message)]`: Indicates that this struct is a message type compatible with the Actix actor framework.
/// - `#[rtype(result = "()")]`: Specifies that the actor handling this message does not need to return any result.
#[derive(actix::Message, Debug)]
#[rtype(result = "()")]
pub struct InvokeScope {
    pub task_scope: TaskScope,
    pub task_spec: TaskSpec,
    pub comms: TaskComms,
    pub task_trigger: TaskTrigger,
    /// A one-shot sender channel to signal the completion of the task processing and provide the resulting task report and its tree structure.
    pub done: tokio::sync::oneshot::Sender<TaskReportAndTree>,
}

impl InvokeScope {
    pub fn new(
        task_scope: TaskScope,
        task_trigger: TaskTrigger,
        task_spec: TaskSpec,
        comms: TaskComms,
        done: tokio::sync::oneshot::Sender<TaskReportAndTree>,
    ) -> Self {
        Self {
            task_scope,
            task_trigger,
            task_spec,
            comms,
            done,
        }
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
struct InsertOutputStream {
    id: usize,
    rx: tokio::sync::mpsc::Receiver<String>,
}

impl actix::Handler<InsertOutputStream> for BsSystem {
    type Result = ();
    fn handle(&mut self, msg: InsertOutputStream, _ctx: &mut Self::Context) -> Self::Result {
        let InsertOutputStream { id: _, rx } = msg;
        let _stream = ReceiverStream::new(rx);
        // <Self as StreamHandler<String>>::add_stream(stream, ctx);
        // dbg!("got it?");
        // self.channels.insert(id, tx);
    }
}

impl Handler<InvokeScope> for BsSystem {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: InvokeScope, _ctx: &mut Self::Context) -> Self::Result {
        let cmd = msg.task_trigger;
        let task_spec = msg.task_spec;
        let task_id = task_spec.as_id();

        let top_level_scope = Box::new(msg.task_scope).into_task_recipient();
        let done = msg.done;
        let comms = msg.comms.clone();
        let tree = task_spec.as_tree();
        let invocation = Invocation {
            id: task_id,
            trigger: cmd,
            comms: comms.clone(),
        };
        let with_start = async move {
            let _sent = comms
                .any_event_sender
                .send(TaskActionStage::started(task_id, tree))
                .await;
            top_level_scope.send(invocation).await
        };
        let next = with_start
            .into_actor(self)
            .map(move |resp, actor, _ctx| match resp {
                Ok(result) => {
                    let report = result.to_report(task_id);
                    let mut e = HashMap::new();
                    every_report(&mut e, &report);

                    let tree = task_spec.as_tree_with_results(&e);
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
