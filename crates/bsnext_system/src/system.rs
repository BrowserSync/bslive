use crate::capabilities::Capabilities;
use crate::fs_task_tracker::FsTaskTracker;
use crate::input_fs::from_input_path;
use crate::invoke_scope::{InvokeScope, Invoker};
use crate::monitor_any::MonitorAny;
use crate::monitor_input::{InputMonitor, MonitorInput};
use crate::path_monitors::PathMonitors;
use crate::tasks::task_spec::TaskSpec;
use actix::{Actor, Addr, AsyncContext, Handler, ResponseFuture, Running};
use actix_rt::Arbiter;
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_core::servers_supervisor::resolve_servers::ResolveServers;
use bsnext_dto::any_event::AnyEvent;
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal_events::InternalEvents;
use bsnext_dto::status_events::{
    server_status_hash, ServerStatusReader, ServerStatusWriter, ServersStatus, StateValue,
};
use bsnext_dto::task_events::TaskReportAndTree;
use bsnext_dto::{InputErrorDetailDTO, StartupErrorDTO};
use bsnext_input::input_fs::ResolvedInputOutcome;
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use bsnext_input::startup::StartupContext;
use bsnext_input::{Input, InputArgs, InputCtx, InputError};
use bsnext_task::task_trigger::{ExecTrigger, TaskTrigger, TaskTriggerSource};
use std::collections::{BTreeSet, HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::Receiver;

#[derive(Debug)]
pub struct BsSystem {
    pub(crate) self_addr: Option<Addr<BsSystem>>,
    servers_addr: Addr<ServersSupervisor>,
    any_event_sender: Sender<AnyEvent>,
    status_tracker: Addr<StatusTracker>,
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
        let status_tracker = StatusTracker::new();
        let status_tracker = status_tracker.start();
        let servers = ServersSupervisor::new(
            tx,
            any_event_sender.clone(),
            status_tracker.clone().recipient(),
            status_tracker.clone().recipient(),
        );
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
            status_tracker,
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

pub struct StatusTracker {
    input: Input,
    servers_status: ServersStatus,
}

impl Actor for StatusTracker {
    type Context = actix::Context<Self>;
}

#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
struct Accept {
    input: Input,
}

impl Handler<Accept> for StatusTracker {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: Accept, _ctx: &mut Self::Context) -> Self::Result {
        self.mark_servers(&msg.input);
        self.input = msg.input;
        tracing::info!(status = ?self.servers_status, "did mark servers, next state");
        Box::pin(async move {
            let a = "";
        })
    }
}

impl StatusTracker {
    pub fn new() -> Self {
        Self {
            servers_status: ServersStatus::default(),
            input: Input::default(),
        }
    }
    fn mark_servers(&mut self, input: &Input) {
        tracing::info!("will mark servers");
        let id = server_status_hash(&input.servers);
        let desired = StateValue::new(id);
        self.servers_status = match self.servers_status {
            ServersStatus::Idle => ServersStatus::Reconciling {
                observed: None,
                desired,
            },
            ServersStatus::Reconciling { .. } => todo!("reconciling"),
            ServersStatus::Ready { observed, .. } if observed != desired => {
                ServersStatus::Reconciling {
                    observed: Some(observed),
                    desired,
                }
            }
            ServersStatus::Ready { observed, desired } => {
                ServersStatus::Ready { observed, desired }
            }
        };
        tracing::info!(id = id, "hasher for servers")
    }
}

impl Handler<ServerStatusReader> for StatusTracker {
    type Result = ResponseFuture<(ServersStatus, Input)>;

    fn handle(&mut self, _msg: ServerStatusReader, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(futures::future::ready((
            self.servers_status,
            self.input.clone(),
        )))
    }
}

impl Handler<ServerStatusWriter> for StatusTracker {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: ServerStatusWriter, _ctx: &mut Self::Context) -> Self::Result {
        tracing::info!(status = ?msg.server_status, "got next server status");
        self.servers_status = msg.server_status;
        Box::pin(async move {})
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
    pub optional_port: Option<u16>,
}

impl ResolveInput {
    pub fn from_strs<A: AsRef<str>>(paths: &[A], optional_port: &Option<u16>) -> Self {
        Self {
            input_paths: paths.iter().map(|s| s.as_ref().to_string()).collect(),
            optional_port: optional_port.clone(),
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
        let optional_port = msg.optional_port;
        Box::pin(async move {
            match ResolvedInputOutcome::new(cwd.clone(), &msg.input_paths) {
                ResolvedInputOutcome::Missing { err, .. } => Err(err),
                ResolvedInputOutcome::GivenPath { ref absolute, .. } => {
                    let args = optional_port.map(InputArgs::new);
                    let ctx = InputCtx::new(&[], args, &start, Some(absolute));
                    Ok(Some(ResolveInputResult {
                        input: from_input_path(absolute, &ctx)?,
                        absolute: absolute.clone(),
                    }))
                }
                ResolvedInputOutcome::Auto { ref absolute, .. } => {
                    let args = optional_port.map(InputArgs::new);
                    let ctx = InputCtx::new(&[], args, &start, Some(absolute));
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
        tracing::info!("will commit input");

        let status_tracker = self.status_tracker.clone();

        // let servers_state = input.servers;
        // todo: make this another thing to eventually come up
        self_addr.do_send(MonitorAny::new(input.clone()));
        Box::pin(async move {
            let _ = status_tracker
                .send(Accept { input })
                .await
                .expect("mailbox");

            tracing::info!("will ping servers");
            servers.do_send(ResolveServers::new());

            tracing::info!("did send accept");
            Ok(())
        })
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

#[derive(Debug)]
pub struct RunOk {
    #[allow(dead_code)]
    pub report_and_tree: TaskReportAndTree,
}

pub struct RunDryOk;
