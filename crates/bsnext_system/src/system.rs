use crate::capabilities::Capabilities;
use crate::fs_task_tracker::FsTaskTracker;
use crate::invoke_scope::{InvokeScope, Invoker};
use crate::run::resolve_spec::{InvokeRunTasks, ResolveSpec};
use crate::servers::ResolveServers;
use crate::tasks::resolve::ResolveInitialTasks;
use crate::tasks::task_spec::TaskSpec;
use crate::watchables::input_monitor::InputMonitor;
use crate::watchables::path_monitor::{PathMonitor, PathMonitorMeta};
use crate::watchables::path_watchable::PathWatchable;
use actix::{Actor, Addr, AsyncContext, ResponseFuture, Running};
use actix_rt::Arbiter;
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_dto::external_events::{ExternalEventsDTO, TaskTreePreview, TaskTreeSummary};
use bsnext_dto::internal::{AnyEvent, ChildResult, TaskReportAndTree};
use bsnext_dto::GetActiveServersResponse;
use bsnext_input::startup::{StartupContext, TopLevelRunMode};
use bsnext_input::Input;
use bsnext_task::task_trigger::{ExecTrigger, TaskTrigger, TaskTriggerSource};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::Receiver;

#[derive(Debug)]
pub struct BsSystem {
    pub(crate) self_addr: Option<Addr<BsSystem>>,
    capabilities_addr: Addr<Capabilities>,
    servers_addr: Addr<ServersSupervisor>,
    any_event_sender: Sender<AnyEvent>,
    pub(crate) input_monitors: Option<InputMonitor>,
    pub(crate) any_monitors: HashMap<PathWatchable, (Addr<PathMonitor>, PathMonitorMeta)>,
    pub(crate) fs_task_tracker: Addr<FsTaskTracker>,
    pub(crate) invoker_addr: Addr<Invoker>,
    pub(crate) cwd: PathBuf,
    pub(crate) start_context: StartupContext,
}

impl Actor for BsSystem {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        tracing::trace!(actor.name = "BsSystem", actor.lifecyle = "started");
        self.self_addr = Some(ctx.address());
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        tracing::trace!(actor.name = "BsSystem", actor.lifecyle = "stopping");
        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(actor.name = "BsSystem", actor.lifecyle = "stopped");
        self.self_addr = None;
    }
}

impl BsSystem {
    pub fn capabilities(&self) -> &Addr<Capabilities> {
        &self.capabilities_addr
    }
    pub fn servers(&self) -> &Addr<ServersSupervisor> {
        &self.servers_addr
    }
    pub fn sender(&self) -> &Sender<AnyEvent> {
        &self.any_event_sender
    }

    pub fn new(
        any_event_sender: Sender<AnyEvent>,
        cwd: PathBuf,
        tx: tokio::sync::oneshot::Sender<()>,
    ) -> Self {
        let servers = ServersSupervisor::new(tx);
        let servers_addr = servers.start();
        let capabilities = Capabilities::new(any_event_sender.clone(), servers_addr.clone());
        let capabilities_addr = capabilities.start();
        let start_context = StartupContext::from_cwd(Some(&cwd));
        let invoker = Invoker::new(
            capabilities_addr.clone(),
            servers_addr.clone(),
            any_event_sender.clone(),
        );
        let invoker_addr = invoker.start();
        let fs_task_tracker = FsTaskTracker::new(invoker_addr.clone().recipient()).start();
        BsSystem {
            self_addr: None,
            capabilities_addr,
            servers_addr,
            any_event_sender,
            input_monitors: None,
            any_monitors: Default::default(),
            invoker_addr,
            fs_task_tracker,
            cwd,
            start_context,
        }
    }

    pub fn publish_any_event(&mut self, evt: AnyEvent) {
        tracing::trace!(?evt);
        let sender = self.any_event_sender.clone();

        Arbiter::current().spawn({
            async move {
                match sender.send(evt).await {
                    Ok(_) => {}
                    Err(_) => tracing::error!("could not send"),
                }
            }
        });
    }

    pub(crate) fn before(&mut self, input: &Input) -> TaskSpec {
        let all = input.before_run_opts();
        TaskSpec::seq_from(&all)
    }

    pub(crate) fn spec_to_invoke_scope(
        &mut self,
        spec: TaskSpec,
    ) -> (InvokeScope, Receiver<TaskReportAndTree>) {
        let trigger = TaskTrigger::new(TaskTriggerSource::Exec(ExecTrigger));

        let (tx, rx) = tokio::sync::oneshot::channel::<TaskReportAndTree>();
        (InvokeScope::new(trigger, spec, tx), rx)
    }
}

#[derive(Debug, actix::Message)]
#[rtype(result = "Result<(), anyhow::Error>")]
pub struct ExternalEventMsg {
    pub evt: ExternalEventsDTO,
}

impl actix::Handler<ExternalEventMsg> for BsSystem {
    type Result = ResponseFuture<Result<(), anyhow::Error>>;
    fn handle(&mut self, msg: ExternalEventMsg, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin({
            let sender = self.any_event_sender.clone();
            async move {
                sender.send(AnyEvent::External(msg.evt)).await?;
                Ok(())
            }
        })
    }
}

pub async fn setup_jobs(addr: Addr<BsSystem>, input: Input) -> anyhow::Result<SetupOk> {
    let clone = input.clone();
    let clone2 = input.clone();

    let spec = addr.send(ResolveInitialTasks::new(clone)).await??;
    let report_and_tree = addr.send(InvokeRunTasks::new(spec)).await??;
    let (servers, child_results) = addr.send(ResolveServers::new(clone2)).await??;
    Ok(SetupOk {
        report_and_tree,
        servers,
        child_results,
    })
}

pub async fn run_jobs(
    addr: Addr<BsSystem>,
    input: Input,
    named: Vec<String>,
    top_level_run_mode: TopLevelRunMode,
    preview: bool,
    summary: bool,
) -> anyhow::Result<RunOk> {
    let spec_output = addr
        .send(ResolveSpec::new(input, named, top_level_run_mode))
        .await??;
    let spec = spec_output.as_spec();
    let tree = spec.as_tree();

    if preview {
        addr.send(ExternalEventMsg {
            evt: ExternalEventsDTO::TaskTreePreview(TaskTreePreview {
                tree: tree.clone(),
                will_exec: true,
            }),
        })
        .await??;
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    let report_and_tree = addr.send(InvokeRunTasks::new(spec.clone())).await??;

    if summary {
        let tree = spec.as_tree();
        addr.send(ExternalEventMsg {
            evt: ExternalEventsDTO::TaskTreeSummary(TaskTreeSummary::from_report(
                tree,
                &report_and_tree.report_map,
            )),
        })
        .await??;
    }

    Ok(RunOk { report_and_tree })
}

pub async fn print_jobs(
    addr: Addr<BsSystem>,
    input: Input,
    named: Vec<String>,
    top_level_run_mode: TopLevelRunMode,
) -> anyhow::Result<RunDryOk> {
    let spec_output = addr
        .send(ResolveSpec::new(input, named, top_level_run_mode))
        .await??;
    let spec = spec_output.as_spec();
    let tree = spec.as_tree();
    addr.send(ExternalEventMsg {
        evt: ExternalEventsDTO::TaskTreePreview(TaskTreePreview {
            tree: tree.clone(),
            will_exec: false,
        }),
    })
    .await??;
    Ok(RunDryOk)
}

pub struct SetupOk {
    pub(crate) servers: GetActiveServersResponse,
    #[allow(dead_code)]
    report_and_tree: TaskReportAndTree,
    pub(crate) child_results: Vec<ChildResult>,
}

pub struct RunOk {
    #[allow(dead_code)]
    pub report_and_tree: TaskReportAndTree,
}

pub struct RunDryOk;
