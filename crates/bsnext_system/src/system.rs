use crate::capabilities::Capabilities;
use crate::fs_task_tracker::FsTaskTracker;
use crate::input_fs::from_input_path;
use crate::invoke_scope::{InvokeScope, Invoker};
use crate::monitor_input::{InputMonitor, MonitorInput};
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
use bsnext_input::startup::StartupContext;
use bsnext_input::{Input, InputArgs, InputCtx, InputError};
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
        let status_tracker = StatusTracker::new(cwd.clone());
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

#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct Accept {
    pub input: Input,
    pub cwd: PathBuf,
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
        let start = self.start_context.clone();
        let optional_port = msg.optional_port;
        Box::pin(async move {
            match ResolvedInputOutcome::new(cwd.clone(), &msg.input_paths) {
                ResolvedInputOutcome::Missing { err, .. } => Err(err),
                ResolvedInputOutcome::GivenPath { ref absolute, .. } => {
                    let args = optional_port.map(InputArgs::new);
                    let ctx = InputCtx::new(&[], args.clone(), &start, Some(absolute));
                    tracing::debug!(?absolute, ?args, ?ctx, "ResolvedInputOutcome::GivenPath");
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
pub(crate) struct ResolveInputAgain;

impl actix::Handler<ResolveInputAgain> for BsSystem {
    type Result = ResponseActFuture<Self, Result<(), anyhow::Error>>;

    fn handle(&mut self, _msg: ResolveInputAgain, ctx: &mut Self::Context) -> Self::Result {
        let Some(monitor) = &self.input_monitors else {
            todo!("this cannot occur?");
        };

        let Some(pb) = monitor.input_ctx.file_path() else {
            todo!("this cannot occur?");
        };

        let input_ctx = monitor.input_ctx.clone();
        let port = input_ctx.port();

        let as_str = pb.to_string_lossy();
        let as_str = as_str.to_string();
        let msg = ResolveInput {
            input_paths: vec![as_str],
            optional_port: port,
        };

        self.publish_external_event(AnyEvent::External(ExternalEventsDTO::InputFileChanged(
            bsnext_dto::FileChangedDTO::from_path_buf(pb),
        )));

        let sys_addr = ctx.address().clone();

        let future = async move {
            let Some(ResolveInputResult { input, absolute }) = sys_addr.send(msg).await?? else {
                todo!("...");
            };

            Ok::<_, anyhow::Error>((input, absolute, input_ctx))
        };

        Box::pin(future.into_actor(self).map(|result, _actor, ctx| {
            if let Ok((input, absolute, input_ctx)) = result {
                ctx.notify(CommitInputFile {
                    input,
                    absolute,
                    ctx: input_ctx,
                });
            } else {
                tracing::error!("something went wrong?")
            }
            Ok(())
        }))
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

    fn handle(&mut self, msg: CommitInput, _ctx: &mut Self::Context) -> Self::Result {
        let input = msg.input;
        let servers = self.servers().clone();
        let monitors = self.path_monitors.clone();
        let cwd = self.cwd.clone();
        tracing::info!("will commit input");

        let status_tracker = self.status_tracker.clone();

        // let servers_state = input.servers;
        // todo: make this another thing to eventually come up
        Box::pin(async move {
            status_tracker
                .send(Accept { input, cwd })
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
