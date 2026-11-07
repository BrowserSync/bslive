use crate::capabilities::Capabilities;
use crate::fs_task_tracker::FsTaskTracker;
use crate::input_fs::from_input_path;
use crate::invoke_scope::{InvokeScope, Invoker};
use crate::path_monitors::{ConfigureRecipient, MonitorAny, PathMonitors};
use crate::status_tracker::StatusTracker;
use crate::tasks::task_spec::TaskSpec;
use actix::ActorFutureExt;
use actix::{
    Actor, Addr, AsyncContext, Handler, ResponseActFuture, ResponseFuture, Running, WrapFuture,
};
use actix_rt::Arbiter;
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_core::servers_supervisor::resolve_servers::ResolveServers;
use bsnext_dto::any_event::AnyEvent;
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal_events::InternalEvents;
use bsnext_dto::task_events::TaskReportAndTree;
use bsnext_dto::{InputErrorDetailDTO, StartupErrorDTO};
use bsnext_input::input_fs::ResolvedInputOutcome;
use bsnext_input::input_src::{FileInputSrc, InputSrc};
use bsnext_input::startup::StartupContext;
use bsnext_input::{Input, InputArgs, InputCtx, InputCtxSource, InputError};
use bsnext_task::task_trigger::{ExecTrigger, TaskTrigger, TaskTriggerSource};
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot::Receiver;

#[derive(Debug)]
pub struct BsSystem {
    pub(crate) self_addr: Option<Addr<BsSystem>>,
    servers_addr: Addr<ServersSupervisor>,
    any_event_sender: Sender<AnyEvent>,
    pub(crate) status_tracker: Addr<StatusTracker>,
    pub(crate) input_ctx: Option<InputCtx>,
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
        self.path_monitors
            .do_send(ConfigureRecipient::new(ctx.address().recipient()));
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
        // todo!("figure where the event goes? probably the status tracker can broadcast updates onto any_event_sender?");
        let status_tracker = StatusTracker::new(cwd.clone(), any_event_sender.clone());
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
        let path_monitors = PathMonitors::new(
            cwd.clone(),
            status_tracker.clone().recipient(),
            status_tracker.clone().recipient(),
        );
        let monitor = path_monitors.start();
        BsSystem {
            self_addr: None,
            servers_addr,
            any_event_sender,
            input_ctx: None,
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

#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct Accept {
    pub input: Input,
    pub cwd: PathBuf,
    pub input_src: Vec<InputSrc>,
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
            optional_port: *optional_port,
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
        let startup_ctx = self.start_context.clone();

        let optional_port = msg
            .optional_port
            .or(self.input_ctx.as_ref().and_then(|x| x.port()));

        let prev_ids = self
            .input_ctx
            .as_ref()
            .and_then(|ctx| ctx.server_ids().map(ToOwned::to_owned))
            .unwrap_or_default();

        let input_paths = if !msg.input_paths.is_empty() {
            msg.input_paths
        } else {
            let prev_path = self
                .input_ctx
                .as_ref()
                .and_then(|c| c.file_path().to_owned());
            prev_path
                .map(|pb| vec![pb.to_string_lossy().to_string()])
                .unwrap_or_default()
        };

        Box::pin(async move {
            match ResolvedInputOutcome::new(cwd.clone(), &input_paths) {
                ResolvedInputOutcome::Missing { err, .. } => Err(err),
                ResolvedInputOutcome::GivenPath { ref absolute, .. } => {
                    let args = optional_port.map(InputArgs::new);
                    let ctx = InputCtx::new(&prev_ids, args.clone(), &startup_ctx, Some(absolute));
                    tracing::debug!(?absolute, ?args, ?ctx, "ResolvedInputOutcome::GivenPath");
                    Ok(Some(ResolveInputResult {
                        input: from_input_path(absolute, &ctx)?,
                        absolute: absolute.clone(),
                    }))
                }
                ResolvedInputOutcome::Auto { ref absolute, .. } => {
                    let args = optional_port.map(InputArgs::new);
                    let ctx = InputCtx::new(&prev_ids, args, &startup_ctx, Some(absolute));
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
pub(crate) struct ResolveInputAgain;

impl actix::Handler<ResolveInputAgain> for BsSystem {
    type Result = ResponseActFuture<Self, Result<(), anyhow::Error>>;

    fn handle(&mut self, _msg: ResolveInputAgain, ctx: &mut Self::Context) -> Self::Result {
        let msg = ResolveInput {
            input_paths: vec![],
            optional_port: None,
        };
        let f = ctx.address().send(msg);
        Box::pin(f.into_actor(self).map(|result, _actor, ctx| {
            match result {
                Ok(Ok(Some(r))) => {
                    ctx.notify(CommitInputFile {
                        input: r.input,
                        absolute: r.absolute,
                        ctx: InputCtxSource::Prev,
                    });
                }
                _ => {
                    tracing::error!("something went wrong?")
                }
            }
            Ok(())
        }))
    }
}

#[derive(Debug, actix::Message)]
#[rtype(result = "Result<(), anyhow::Error>")]
pub struct CommitInput {
    pub(crate) input: Input,
    pub(crate) input_src: Vec<InputSrc>,
}

impl CommitInput {
    pub fn new(input: impl Into<Input>, srcs: &[InputSrc]) -> Self {
        Self {
            input: input.into(),
            input_src: srcs.to_vec(),
        }
    }
}

impl actix::Handler<CommitInput> for BsSystem {
    type Result = ResponseFuture<Result<(), anyhow::Error>>;

    fn handle(&mut self, msg: CommitInput, _ctx: &mut Self::Context) -> Self::Result {
        let input = msg.input;
        let input_src = msg.input_src;
        let servers = self.servers().clone();
        let monitors = self.path_monitors.clone();
        let cwd = self.cwd.clone();
        tracing::info!("will commit input");

        let status_tracker = self.status_tracker.clone();

        // let servers_state = input.servers;
        // todo: make this another thing to eventually come up
        Box::pin(async move {
            status_tracker
                .send(Accept {
                    input,
                    cwd,
                    input_src,
                })
                .await
                .expect("mailbox");

            tracing::info!("will ping servers");
            servers.do_send(ResolveServers::new());
            monitors.do_send(MonitorAny::new());

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
    pub(crate) ctx: InputCtxSource,
}

impl CommitInputFile {
    pub fn new(input: impl Into<Input>, pb: impl Into<PathBuf>, ctx: InputCtxSource) -> Self {
        Self {
            input: input.into(),
            absolute: pb.into(),
            ctx,
        }
    }
}

impl actix::Handler<CommitInputFile> for BsSystem {
    type Result = ResponseFuture<Result<(), anyhow::Error>>;

    fn handle(&mut self, msg: CommitInputFile, ctx: &mut Self::Context) -> Self::Result {
        let input = msg.input;
        let self_addr = ctx.address();
        let abs = msg.absolute;
        match msg.ctx {
            InputCtxSource::Prev => {
                // noop
            }
            InputCtxSource::New(ctx) => self.input_ctx = Some(ctx),
        };
        Box::pin(async move {
            let _v = self_addr
                .send(CommitInput {
                    input,
                    input_src: vec![InputSrc::File(FileInputSrc { absolute: abs })],
                })
                .await?;
            // self_addr.send(MonitorInput::new(abs, ctx)).await?;
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
