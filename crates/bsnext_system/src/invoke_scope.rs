use crate::capabilities::Capabilities;
use crate::tasks::task_spec::TaskSpec;
use actix::ActorFutureExt;
use actix::Handler;
use actix::WrapFuture;
use actix::{Actor, Addr, ResponseActFuture};
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_dto::internal::{AnyEvent, TaskActionStage, TaskReportAndTree};
use bsnext_task::as_actor::AsActor;
use bsnext_task::invocation::Invocation;
use bsnext_task::task_trigger::TaskTrigger;
use std::time::Duration;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Invoker {
    capabilities_addr: Addr<Capabilities>,
    servers_addr: Addr<ServersSupervisor>,
    any_event_sender: Sender<AnyEvent>,
}

impl Invoker {
    pub fn new(
        capabilities_addr: Addr<Capabilities>,
        servers_addr: Addr<ServersSupervisor>,
        any_event_sender: Sender<AnyEvent>,
    ) -> Self {
        Self {
            capabilities_addr,
            servers_addr,
            any_event_sender,
        }
    }
}

impl Actor for Invoker {
    type Context = actix::Context<Self>;
}

/// A struct representing a message to trigger a specific task in the system.
/// This message will be handled by an actor in the Actix framework.
///
/// # Derive Attributes
/// - `#[derive(actix::Message)]`: Indicates that this struct is a message type compatible with the Actix actor framework.
/// - `#[rtype(result = "()")]`: Specifies that the actor handling this message does not need to return any result.
#[derive(actix::Message, Debug)]
#[rtype(result = "()")]
pub struct InvokeScope {
    pub task_spec: TaskSpec,
    pub task_trigger: TaskTrigger,
    /// A one-shot sender channel to signal the completion of the task processing and provide the resulting task report and its tree structure.
    pub done: tokio::sync::oneshot::Sender<TaskReportAndTree>,
}

impl InvokeScope {
    pub fn new(
        task_trigger: TaskTrigger,
        task_spec: TaskSpec,
        done: tokio::sync::oneshot::Sender<TaskReportAndTree>,
    ) -> Self {
        Self {
            task_trigger,
            task_spec,
            done,
        }
    }
}

impl Handler<InvokeScope> for Invoker {
    type Result = ResponseActFuture<Self, ()>;

    #[tracing::instrument(skip_all, name = "InvokeScope", fields(path = %msg.task_spec.path()))]
    fn handle(&mut self, msg: InvokeScope, _ctx: &mut Self::Context) -> Self::Result {
        let trigger = msg.task_trigger;
        let task_spec = msg.task_spec;
        let task_spec_clone = task_spec.clone();
        let node_path = task_spec.path().to_owned();
        let tree = task_spec.as_tree();
        let scope =
            task_spec.to_task_scope(self.servers_addr.clone(), self.capabilities_addr.clone());
        let top_level_scope = Box::new(scope).into_task_recipient();
        let done = msg.done;
        let c1 = self.any_event_sender.clone();
        let invocation = Invocation::new(&node_path, trigger);

        let with_start = async move {
            let _started = c1.send(TaskActionStage::started(tree)).await;
            let result = top_level_scope.send(invocation).await?;
            let (report, report_map) = result.to_report_and_map(node_path);
            let tree = task_spec_clone.as_tree_with_results(&report_map);
            let report_and_tree = TaskReportAndTree {
                report: report.clone(),
                tree: tree.clone(),
                report_map,
            };

            // mark complete
            match c1
                .send_timeout(
                    TaskActionStage::complete(tree, report),
                    Duration::from_secs(1),
                )
                .await
            {
                Ok(_) => tracing::debug!("did send completion"),
                Err(_e) => tracing::error!(
                    "failed to send task completion event: timeout or channel closed"
                ),
            }

            match done.send(report_and_tree) {
                Ok(_) => tracing::debug!("did finish initial"),
                Err(_) => tracing::error!("failed to send task report and tree: receiver dropped"),
            };

            Ok::<(), _>(())
        };

        Box::pin(
            with_start
                .into_actor(self)
                .map(|_res: anyhow::Result<()>, _actor, _ctx| ()),
        )
    }
}
