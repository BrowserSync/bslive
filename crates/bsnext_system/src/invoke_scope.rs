use crate::system::BsSystem;
use crate::tasks::task_comms::TaskComms;
use crate::tasks::task_spec::TaskSpec;
use actix::ActorFutureExt;
use actix::Handler;
use actix::ResponseActFuture;
use actix::WrapFuture;
use bsnext_dto::internal::{TaskActionStage, TaskReportAndTree};
use bsnext_task::as_actor::AsActor;
use bsnext_task::invocation::{Invocation, SpecId};
use bsnext_task::task_scope::TaskScope;
use bsnext_task::task_trigger::TaskTrigger;
use bsnext_task::ContentId;

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

impl Handler<InvokeScope> for BsSystem {
    type Result = ResponseActFuture<Self, ()>;

    #[tracing::instrument(skip_all, name = "InvokeScope")]
    fn handle(&mut self, msg: InvokeScope, _ctx: &mut Self::Context) -> Self::Result {
        let task_trigger = msg.task_trigger;
        let task_spec = msg.task_spec;
        let spec_id_raw = task_spec.as_id();
        let spec_id = SpecId::new(ContentId::new(spec_id_raw));

        let top_level_scope = Box::new(msg.task_scope).into_task_recipient();
        let done = msg.done;
        let comms = msg.comms.clone();
        let tree = task_spec.as_tree();
        let invocation = Invocation::new(spec_id, task_trigger);
        let with_start = async move {
            let _sent = comms
                .any_event_sender
                .send(TaskActionStage::started(tree))
                .await;
            top_level_scope.send(invocation).await
        };
        let next = with_start
            .into_actor(self)
            .map(move |resp, actor, _ctx| match resp {
                Ok(result) => {
                    let (report, report_map) = result.to_report_and_map(spec_id);
                    let tree = task_spec.as_tree_with_results(&report_map);
                    let report_and_tree = TaskReportAndTree {
                        report: report.clone(),
                        tree: tree.clone(),
                        report_map,
                    };
                    actor.publish_any_event(TaskActionStage::complete(tree, report));
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
