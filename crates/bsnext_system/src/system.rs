use crate::any_watchable::to_any_watchables;
use crate::capabilities::Capabilities;
use crate::input_monitor::InputMonitor;
use crate::invoke_scope::InvokeScope;
use crate::monitor_path_watchables::MonitorPathWatchables;
use crate::path_monitor::{PathMonitor, PathMonitorMeta};
use crate::path_watchable::PathWatchable;
use crate::route_watchable::to_route_watchables;
use crate::run::resolve_run::{InvokeRunTasks, ResolveRunTasks};
use crate::server_watchable::to_server_watchables;
use crate::servers::ResolveServers;
use crate::tasks::resolve::ResolveInitialTasks;
use crate::tasks::task_spec::TaskSpec;
use actix::{Actor, Addr, AsyncContext, Running};
use actix_rt::Arbiter;
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_dto::archy::ArchyNode;
use bsnext_dto::internal::{AnyEvent, ChildResult, TaskReportAndTree};
use bsnext_dto::GetActiveServersResponse;
use bsnext_fs::FsEventContext;
use bsnext_input::startup::{StartupContext, TopLevelRunMode};
use bsnext_input::Input;
use bsnext_task::task_trigger::{TaskTrigger, TaskTriggerSource};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::Receiver;
use tracing::debug;

#[derive(Debug)]
pub struct BsSystem {
    pub(crate) self_addr: Option<Addr<BsSystem>>,
    capabilities_addr: Addr<Capabilities>,
    servers_addr: Addr<ServersSupervisor>,
    any_event_sender: Sender<AnyEvent>,
    pub(crate) input_monitors: Option<InputMonitor>,
    pub(crate) any_monitors: HashMap<PathWatchable, (Addr<PathMonitor>, PathMonitorMeta)>,
    pub(crate) task_spec_mapping: HashMap<FsEventContext, TaskSpec>,
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
        // self.servers_addr = None;
        // self.any_event_sender = None;
    }
}

impl BsSystem {
    pub fn capabilities(&self) -> Addr<Capabilities> {
        self.capabilities_addr.clone()
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
        let capabilities = Capabilities::new(any_event_sender.clone());
        let start_context = StartupContext::from_cwd(Some(&cwd));
        BsSystem {
            self_addr: None,
            capabilities_addr: capabilities.start(),
            servers_addr: servers.start(),
            any_event_sender,
            input_monitors: None,
            any_monitors: Default::default(),
            task_spec_mapping: Default::default(),
            cwd,
            start_context,
        }
    }

    #[tracing::instrument(skip_all, name = "BsSystem.accept_watchables")]
    pub(crate) fn accept_watchables(&mut self, input: &Input) {
        let route_watchables = to_route_watchables(input);
        let server_watchables = to_server_watchables(input);
        let any_watchables = to_any_watchables(input);

        debug!("processing {} route watchables", route_watchables.len(),);
        debug!("processing {} server watchables", server_watchables.len());
        debug!("processing {} any watchables", any_watchables.len());

        let Some(self_address) = &self.self_addr else {
            unreachable!("?")
        };

        // todo: clean up this merging
        let all_watchables = route_watchables
            .iter()
            .map(|r| PathWatchable::Route(r.to_owned()));

        let servers = server_watchables
            .iter()
            .map(|w| PathWatchable::Server(w.to_owned()));

        let any = any_watchables
            .iter()
            .map(|w| PathWatchable::Any(w.to_owned()));

        let watchables = all_watchables.chain(servers).chain(any).collect::<Vec<_>>();

        let cwd = self.cwd.clone();
        let addr = self_address.clone();
        debug!(
            "{} watchables to add, cwd: {}",
            watchables.len(),
            cwd.display()
        );
        let msg = MonitorPathWatchables { watchables, cwd };

        addr.do_send(msg);
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

    pub(crate) fn before(
        &mut self,
        input: &Input,
        addr: Addr<Capabilities>,
    ) -> (InvokeScope, Receiver<TaskReportAndTree>) {
        let comms = self.task_comms();
        let all = input.before_run_opts();
        let task_spec = TaskSpec::seq_from(&all);

        // let tree = task_spec.as_tree();
        // let next = archy(&tree, Prefix::None);
        // print!("{next}");

        let trigger = TaskTrigger::new(TaskTriggerSource::Exec, 0);

        debug!("{} before tasks to execute", all.len());
        let task_scope = task_spec
            .clone()
            .to_task_scope(self.servers_addr.clone(), addr);
        let (tx, rx) = tokio::sync::oneshot::channel::<TaskReportAndTree>();
        (
            InvokeScope::new(task_scope, trigger, task_spec, comms, tx),
            rx,
        )
    }

    pub(crate) fn run_only(
        &mut self,
        addr: Addr<Capabilities>,
        spec: TaskSpec,
    ) -> (InvokeScope, Receiver<TaskReportAndTree>) {
        let comms = self.task_comms();
        let trigger = TaskTrigger::new(TaskTriggerSource::Exec, 0);

        let task_scope = spec.clone().to_task_scope(self.servers_addr.clone(), addr);
        let (tx, rx) = tokio::sync::oneshot::channel::<TaskReportAndTree>();
        (InvokeScope::new(task_scope, trigger, spec, comms, tx), rx)
    }
}

pub async fn setup_jobs(addr: Addr<BsSystem>, input: Input) -> anyhow::Result<SetupOk> {
    let clone = input.clone();
    let clone2 = input.clone();
    let report_and_tree = addr.send(ResolveInitialTasks { input: clone }).await??;
    let servers_resp = addr.send(ResolveServers { input: clone2 });
    let (servers, child_results) = servers_resp.await??;
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
) -> anyhow::Result<RunOk> {
    let spec_output = addr
        .send(ResolveRunTasks::new(input, named, top_level_run_mode))
        .await??;
    let report_and_tree = addr
        .send(InvokeRunTasks::new(spec_output.task_spec))
        .await??;

    Ok(RunOk { report_and_tree })
}

pub async fn print_jobs(
    addr: Addr<BsSystem>,
    input: Input,
    named: Vec<String>,
    top_level_run_mode: TopLevelRunMode,
) -> anyhow::Result<RunDryOk> {
    let spec_output = addr
        .send(ResolveRunTasks::new(input, named, top_level_run_mode))
        .await??;
    let spec = spec_output.task_spec;
    let tree = spec.as_tree();
    Ok(RunDryOk { tree, spec })
}

pub struct SetupOk {
    pub(crate) servers: GetActiveServersResponse,
    #[allow(dead_code)]
    report_and_tree: TaskReportAndTree,
    pub(crate) child_results: Vec<ChildResult>,
}

pub struct RunOk {
    #[allow(dead_code)]
    report_and_tree: TaskReportAndTree,
}

pub struct RunDryOk {
    #[allow(dead_code)]
    pub tree: ArchyNode,
    pub spec: TaskSpec,
}
