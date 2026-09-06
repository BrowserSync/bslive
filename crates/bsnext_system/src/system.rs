use crate::capabilities::Capabilities;
use crate::fs_task_tracker::FsTaskTracker;
use crate::input_fs::from_input_path;
use crate::invoke_scope::{InvokeScope, Invoker};
use crate::monitor_any::MonitorAny;
use crate::monitor_input::{InputMonitor, MonitorInput};
use crate::path_monitors::PathMonitors;
use crate::run::resolve_spec::{InvokeRunTasks, ResolveSpec};
use crate::tasks::task_spec::TaskSpec;
use actix::{Actor, Addr, AsyncContext, Handler, ResponseFuture, Running};
use actix_rt::Arbiter;
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_core::servers_supervisor::resolve_servers::ResolveServers;
use bsnext_dto::any_event::AnyEvent;
use bsnext_dto::external_events::{ExternalEventsDTO, TaskTreePreview, TaskTreeSummary};
use bsnext_dto::internal_events::InternalEvents;
use bsnext_dto::server_events::ChildResult;
use bsnext_dto::task_events::TaskReportAndTree;
use bsnext_dto::{GetActiveServersResponse, InputErrorDetailDTO, StartupErrorDTO};
use bsnext_input::input_fs::ResolvedInputOutcome;
use bsnext_input::startup::{StartupContext, TopLevelRunMode};
use bsnext_input::{Input, InputCtx, InputError};
use bsnext_task::task_trigger::{ExecTrigger, TaskTrigger, TaskTriggerSource};
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::Receiver;

#[derive(Debug)]
pub struct BsSystem {
    pub(crate) self_addr: Option<Addr<BsSystem>>,
    servers_addr: Addr<ServersSupervisor>,
    any_event_sender: Sender<AnyEvent>,
    pub(crate) input_monitors: Option<InputMonitor>,
    pub(crate) fs_task_tracker: Addr<FsTaskTracker>,
    pub(crate) path_monitors: Addr<PathMonitors>,
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
        let servers = ServersSupervisor::new(tx, any_event_sender.clone());
        let servers_addr = servers.start();
        let capabilities = Capabilities::new(any_event_sender.clone(), servers_addr.clone());
        let capabilities_addr = capabilities.start();
        let start_context = StartupContext::from_cwd(Some(&cwd));
        let invoker = Invoker::new(capabilities_addr.clone(), any_event_sender.clone());
        let invoker_addr = invoker.start();
        let fs_task_tracker = FsTaskTracker::new(invoker_addr.clone().recipient()).start();
        let monitor = PathMonitors::new();
        let monitor = monitor.start();
        BsSystem {
            self_addr: None,
            servers_addr,
            any_event_sender,
            input_monitors: None,
            path_monitors: monitor,
            invoker_addr,
            fs_task_tracker,
            cwd,
            start_context,
        }
    }

    pub fn publish_external_event(&mut self, evt: AnyEvent) {
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

    pub fn publish_internal_event(&mut self, evt: InternalEvents) {
        match evt {
            InternalEvents::InputError(InputError::BsLiveRules(bs_rules)) => {
                let n = miette::GraphicalReportHandler::new();
                let mut inner = String::new();
                n.render_report(&mut inner, &bs_rules).expect("write?");
                let evt = ExternalEventsDTO::InputError(InputErrorDetailDTO { error: inner });
                self.publish_external_event(AnyEvent::External(evt));
            }
            InternalEvents::InputError(err) => {
                let evt = ExternalEventsDTO::InputError(InputErrorDetailDTO {
                    error: err.to_string(),
                });
                self.publish_external_event(AnyEvent::External(evt));
            }
            InternalEvents::StartupError(startup) => {
                let evt = ExternalEventsDTO::StartupError(StartupErrorDTO {
                    error: startup.to_string(),
                });
                self.publish_external_event(AnyEvent::External(evt));
            }
        }
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
#[rtype(result = "StartupContext")]
pub struct GetStartContext;

impl Handler<GetStartContext> for BsSystem {
    type Result = ResponseFuture<StartupContext>;

    fn handle(&mut self, _msg: GetStartContext, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(futures::future::ready(self.start_context.clone()))
    }
}

#[derive(Debug, actix::Message)]
#[rtype(result = "Result<Option<ResolveInputResult>, Box<InputError>>")]
pub struct ResolveInput {
    pub input_paths: Vec<String>,
}

impl ResolveInput {
    pub fn from_strs<A: AsRef<str>>(paths: &[A]) -> Self {
        Self {
            input_paths: paths.iter().map(|s| s.as_ref().to_string()).collect(),
        }
    }
}

#[derive(Debug)]
pub struct ResolveInputResult {
    pub input: Input,
    pub absolute: PathBuf,
}

impl Handler<ResolveInput> for BsSystem {
    type Result = ResponseFuture<Result<Option<ResolveInputResult>, Box<InputError>>>;

    fn handle(&mut self, msg: ResolveInput, _ctx: &mut Self::Context) -> Self::Result {
        let cwd = self.cwd.clone();
        let start = self.start_context.clone();
        Box::pin(async move {
            match ResolvedInputOutcome::new(cwd.clone(), &msg.input_paths) {
                ResolvedInputOutcome::Missing { err, .. } => Err(err),
                ResolvedInputOutcome::GivenPath { ref absolute, .. } => {
                    let ctx = InputCtx::new(&[], None, &start, Some(absolute));
                    Ok(Some(ResolveInputResult {
                        input: from_input_path(absolute, &ctx)?,
                        absolute: absolute.clone(),
                    }))
                }
                ResolvedInputOutcome::Auto { ref absolute, .. } => {
                    let ctx = InputCtx::new(&[], None, &start, Some(absolute));
                    Ok(Some(ResolveInputResult {
                        input: from_input_path(absolute, &ctx)?,
                        absolute: absolute.clone(),
                    }))
                }
                ResolvedInputOutcome::Empty => Ok(None),
            }
        })
    }
}

#[derive(Debug, actix::Message)]
#[rtype(result = "Result<(), anyhow::Error>")]
pub struct CommitInput {
    pub(crate) input: Input,
}

impl CommitInput {
    pub fn new(input: impl Into<Input>) -> Self {
        Self {
            input: input.into(),
        }
    }
}

impl actix::Handler<CommitInput> for BsSystem {
    type Result = ResponseFuture<Result<(), anyhow::Error>>;

    fn handle(&mut self, msg: CommitInput, ctx: &mut Self::Context) -> Self::Result {
        let input = msg.input;
        let servers = self.servers().clone();
        let self_addr = ctx.address();
        servers.do_send(ResolveServers::new(input.clone()));
        self_addr.do_send(MonitorAny::new(input));
        Box::pin(async move { Ok(()) })
    }
}

#[derive(Debug, actix::Message)]
#[rtype(result = "Result<(), anyhow::Error>")]
pub struct CommitInputFile {
    pub(crate) input: Input,
    pub(crate) absolute: PathBuf,
    pub(crate) ctx: InputCtx,
}

impl CommitInputFile {
    pub fn new(input: impl Into<Input>, pb: impl Into<PathBuf>, ctx: impl Into<InputCtx>) -> Self {
        Self {
            input: input.into(),
            absolute: pb.into(),
            ctx: ctx.into(),
        }
    }
}

impl actix::Handler<CommitInputFile> for BsSystem {
    type Result = ResponseFuture<Result<(), anyhow::Error>>;

    fn handle(&mut self, msg: CommitInputFile, ctx: &mut Self::Context) -> Self::Result {
        let input = msg.input;
        let self_addr = ctx.address();
        let abs = msg.absolute;
        let ctx = msg.ctx;
        Box::pin(async move {
            let _v = self_addr.send(CommitInput { input }).await?;
            self_addr.send(MonitorInput::new(abs, ctx)).await?;
            Ok(())
        })
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

// pub async fn setup_jobs(
//     addr: Addr<BsSystem>,
//     servers_addr: Addr<ServersSupervisor>,
//     input: Input,
// ) -> anyhow::Result<SetupOk> {
//     let clone = input.clone();
//     let clone2 = input.clone();
//
//     let spec = addr.send(ResolveInitialTasks::new(clone)).await??;
//     let report_and_tree = addr.send(InvokeRunTasks::new(spec)).await??;
//     let (servers, child_results) = servers_addr.send(ResolveServers::new(clone2)).await??;
//     Ok(SetupOk {
//         input,
//         report_and_tree,
//         servers,
//         child_results,
//     })
// }

// pub async fn setup_jobs_only(addr: &Addr<BsSystem>, input: Input) -> anyhow::Result<SetupTasksOk> {
//     let spec = addr.send(ResolveInitialTasks::new(input)).await??;
//     let report_and_tree = addr.send(InvokeRunTasks::new(spec)).await??;
//     Ok(SetupTasksOk { report_and_tree })
// }

// pub async fn setup_servers_only(
//     servers_addr: &Addr<ServersSupervisor>,
//     input: Input,
// ) -> anyhow::Result<SetupServersOk> {
//     let (servers, child_results) = servers_addr.send(ResolveServers::new(input)).await??;
//     Ok(SetupServersOk {
//         servers,
//         child_results,
//     })
// }

pub struct SetupOk {
    pub(crate) input: Input,
    pub(crate) servers: GetActiveServersResponse,
    #[allow(dead_code)]
    pub report_and_tree: TaskReportAndTree,
    pub(crate) child_results: Vec<ChildResult>,
}

pub struct SetupServersOk {
    pub(crate) servers: GetActiveServersResponse,
    pub(crate) child_results: Vec<ChildResult>,
}

#[derive(Debug)]
pub struct SetupTasksOk {
    #[allow(dead_code)]
    pub report_and_tree: TaskReportAndTree,
}

#[derive(Debug)]
pub struct RunOk {
    #[allow(dead_code)]
    pub report_and_tree: TaskReportAndTree,
}

pub struct RunDryOk;
