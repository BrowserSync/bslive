use actix::{
    Actor, ActorContext, ActorFutureExt, Addr, AsyncContext, Handler, ResponseActFuture,
    ResponseFuture, Running, WrapFuture, WrapStream,
};

use crate::any_watchable::to_any_watchables;
use crate::monitor_path_watchables::MonitorPathWatchables;
use crate::path_monitor::{PathMonitor, PathMonitorMeta};
use crate::path_watchable::PathWatchable;
use crate::run::resolve_run::{InvokeRunTasks, ResolveRunTasks};
use crate::tasks::task_spec::TaskSpec;
use actix_rt::Arbiter;
use bsnext_core::server::handler_client_config::ClientConfigChange;
use bsnext_core::server::handler_routes_updated::RoutesUpdated;
use bsnext_core::servers_supervisor::actor::{ChildHandler, ChildStopped, ServersSupervisor};
use bsnext_core::servers_supervisor::get_servers_handler::GetActiveServers;
use bsnext_core::servers_supervisor::input_changed_handler::InputChanged;
use bsnext_core::servers_supervisor::start_handler::ChildCreatedInsert;
use bsnext_dto::archy::ArchyNode;
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal::InternalEvents::TaskSpecDisplay;
use bsnext_dto::internal::{
    AnyEvent, ChildResult, InitialTaskError, InternalEvents, ServerError, TaskReportAndTree,
};
use bsnext_dto::{ActiveServer, DidStart, GetActiveServersResponse, StartupError};
use bsnext_fs::{FsEvent, FsEventContext, FsEventGrouping};
use bsnext_input::startup::{
    RunMode, StartupContext, SystemStart, SystemStartArgs, TopLevelRunMode,
};
use bsnext_input::{Input, InputCtx};
use bsnext_task::task_trigger::{TaskTrigger, TaskTriggerSource};
use input_monitor::{InputMonitor, MonitorInput};
use invoke_scope::InvokeScope;
use route_watchable::to_route_watchables;
use server_watchable::to_server_watchables;
use start::start_kind::StartKind;
use std::collections::HashMap;
use std::future::ready;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;
use tracing::{debug, Instrument, Span};

pub mod any_watchable;
pub mod args;
pub mod cli;
pub mod export;
mod external_event_sender;
mod handle_fs_event_grouping;
pub mod input_fs;
mod input_monitor;
mod invoke_scope;
mod monitor_path_watchables;
mod path_monitor;
mod path_watchable;
mod route_watchable;
pub mod run;
pub mod server_watchable;
pub mod start;
pub mod tasks;
mod trigger_fs_task;
pub mod watch;

#[derive(Debug)]
pub struct BsSystem {
    self_addr: Option<Addr<BsSystem>>,
    servers_addr: Option<Addr<ServersSupervisor>>,
    any_event_sender: Option<Sender<AnyEvent>>,
    input_monitors: Option<InputMonitor>,
    any_monitors: HashMap<PathWatchable, (Addr<PathMonitor>, PathMonitorMeta)>,
    task_spec_mapping: HashMap<FsEventContext, TaskSpec>,
    cwd: Option<PathBuf>,
    start_context: Option<StartupContext>,
}

#[derive(Debug)]
pub struct BsSystemApi {
    sys_address: Addr<BsSystem>,
    handle: oneshot::Receiver<()>,
}

impl BsSystemApi {
    ///
    /// Stop the system entirely. Note: this consumes self
    /// and you cannot interact with the system
    ///
    pub async fn stop(&self) -> anyhow::Result<()> {
        self.sys_address
            .send(StopSystem)
            .await
            .map_err(|e| anyhow::anyhow!("could not stop: {:?}", e))
    }
    ///
    /// Use this to keep the server open
    ///
    pub async fn handle(self) -> anyhow::Result<()> {
        self.handle
            .await
            .map_err(|e| anyhow::anyhow!("could not wait: {:?}", e))
    }

    pub fn fs_event(&self, evt: FsEvent) {
        self.sys_address.do_send(FsEventGrouping::Singular(evt))
    }

    pub async fn active_servers(&self) -> Result<Vec<ActiveServer>, ServerError> {
        match self.sys_address.send(ReadActiveServers).await {
            Ok(Ok(resp)) => Ok(resp.servers),
            _ => {
                tracing::error!("Could not send ReadActiveServers to sys_address");
                Err(ServerError::Unknown(
                    "Could not send ReadActiveServers to sys_address".to_string(),
                ))
            }
        }
    }
}

#[derive(Debug)]
pub struct EventWithSpan {
    pub evt: ExternalEventsDTO,
}

impl Default for BsSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl BsSystem {
    pub fn new() -> Self {
        BsSystem {
            self_addr: None,
            servers_addr: None,
            any_event_sender: None,
            input_monitors: None,
            any_monitors: Default::default(),
            task_spec_mapping: Default::default(),
            cwd: None,
            start_context: None,
        }
    }

    #[tracing::instrument(skip_all, name = "BsSystem.accept_watchables")]
    fn accept_watchables(&mut self, input: &Input) {
        let route_watchables = to_route_watchables(input);
        let server_watchables = to_server_watchables(input);
        let any_watchables = to_any_watchables(input);

        debug!("processing {} route watchables", route_watchables.len(),);
        debug!("processing {} server watchables", server_watchables.len());
        debug!("processing {} any watchables", any_watchables.len());

        let Some(self_address) = &self.self_addr else {
            unreachable!("?")
        };

        let Some(cwd) = &self.cwd else {
            unreachable!("can this occur?")
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

        let cwd = cwd.clone();
        let addr = self_address.clone();
        debug!(
            "{} watchables to add, cwd: {}",
            watchables.len(),
            cwd.display()
        );
        let msg = MonitorPathWatchables { watchables, cwd };

        addr.do_send(msg);
    }

    fn update_ctx(&mut self, input: &Input, ctx: &StartupContext) {
        let next = input
            .servers
            .iter()
            .map(|s| s.identity.clone())
            .collect::<Vec<_>>();

        if let Some(mon) = self.input_monitors.as_mut() {
            let next_input_ctx = InputCtx::new(&next, None, ctx, mon.input_ctx.file_path());
            if !next.is_empty() {
                if next_input_ctx == mon.input_ctx {
                    tracing::info!(
                        " - server identities were equal, not updating ctx {:?}",
                        next_input_ctx
                    );
                } else {
                    tracing::info!(
                        " + updating stored server identities following a file change {:?}",
                        next
                    );
                    mon.input_ctx = next_input_ctx
                }
            }
        }
    }

    fn publish_any_event(&mut self, evt: AnyEvent) {
        tracing::trace!(?evt);

        if let Some(any_event_sender) = &self.any_event_sender {
            Arbiter::current().spawn({
                let events_sender = any_event_sender.clone();
                async move {
                    match events_sender.send(evt).await {
                        Ok(_) => {}
                        Err(_) => tracing::error!("could not send"),
                    }
                }
            });
        }
    }

    fn before(
        &mut self,
        input: &Input,
        addr: Addr<BsSystem>,
    ) -> (InvokeScope, Receiver<TaskReportAndTree>) {
        let comms = self.task_comms();
        let all = input.before_run_opts();
        let task_spec = TaskSpec::seq_from(&all);

        // let tree = task_spec.as_tree();
        // let next = archy(&tree, Prefix::None);
        // print!("{next}");

        let trigger = TaskTrigger::new(TaskTriggerSource::Exec, 0);

        debug!("{} before tasks to execute", all.len());
        let task_scope = task_spec.clone().to_task_scope(None, addr);
        let (tx, rx) = tokio::sync::oneshot::channel::<TaskReportAndTree>();
        (
            InvokeScope::new(task_scope, trigger, task_spec, comms, tx),
            rx,
        )
    }

    fn run_only(
        &mut self,
        addr: Addr<BsSystem>,
        spec: TaskSpec,
    ) -> (InvokeScope, Receiver<TaskReportAndTree>) {
        let comms = self.task_comms();
        let trigger = TaskTrigger::new(TaskTriggerSource::Exec, 0);

        let task_scope = spec.clone().to_task_scope(None, addr);
        let (tx, rx) = tokio::sync::oneshot::channel::<TaskReportAndTree>();
        (InvokeScope::new(task_scope, trigger, spec, comms, tx), rx)
    }
}

async fn setup_jobs(addr: Addr<BsSystem>, input: Input) -> anyhow::Result<SetupOk> {
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

async fn run_jobs(
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

async fn print_jobs(
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

struct SetupOk {
    servers: GetActiveServersResponse,
    #[allow(dead_code)]
    report_and_tree: TaskReportAndTree,
    child_results: Vec<ChildResult>,
}

struct RunOk {
    #[allow(dead_code)]
    report_and_tree: TaskReportAndTree,
}

struct RunDryOk {
    #[allow(dead_code)]
    tree: ArchyNode,
    spec: TaskSpec,
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
        self.servers_addr = None;
        self.any_event_sender = None;
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct StopSystem;

impl Handler<StopSystem> for BsSystem {
    type Result = ();

    fn handle(&mut self, _msg: StopSystem, ctx: &mut Self::Context) -> Self::Result {
        tracing::trace!("handling StopSystem. Note: not graceful.");
        ctx.stop();
    }
}

#[derive(Debug, actix::Message)]
#[rtype(result = "Result<DidStart, StartupError>")]
pub struct Start {
    pub kind: StartKind,
    pub cwd: PathBuf,
    pub ack: oneshot::Sender<()>,
    pub events_sender: Sender<AnyEvent>,
}

impl Handler<Start> for BsSystem {
    type Result = ResponseActFuture<Self, Result<DidStart, StartupError>>;

    #[tracing::instrument(name = "BsSystem->Start", skip(self, msg, ctx))]
    fn handle(&mut self, msg: Start, ctx: &mut Self::Context) -> Self::Result {
        self.any_event_sender = Some(msg.events_sender.clone());
        self.cwd = Some(msg.cwd);

        debug!("self.cwd {:?}", self.cwd);

        let Some(cwd) = &self.cwd else {
            unreachable!("?")
        };

        let servers = ServersSupervisor::new(msg.ack);
        // store the servers addr for later
        self.servers_addr = Some(servers.start());

        let start_context = StartupContext::from_cwd(self.cwd.as_ref());
        self.start_context = Some(start_context.clone());

        debug!(?start_context);

        match msg.kind.input(&start_context) {
            Ok(SystemStartArgs::PathWithInput { path, input }) => {
                debug!("SystemStartArgs::PathWithInput");

                let ids = input.ids();
                let input_ctx = InputCtx::new(&ids, None, &start_context, Some(&path));
                let input_clone2 = input.clone();
                let addr = ctx.address();
                let jobs = setup_jobs(addr, input.clone()).instrument(Span::current());

                Box::pin(jobs.into_actor(self).map(
                    move |res: Result<SetupOk, anyhow::Error>, actor, ctx| {
                        let SetupOk { servers, .. } = res.map_err(StartupError::Any)?;
                        debug!("✅ setup jobs completed");
                        ctx.notify(MonitorInput {
                            path: path.clone(),
                            cwd: actor.cwd.clone().unwrap(),
                            input_ctx,
                        });
                        // todo: where to better sequence these side-effects?
                        actor.accept_watchables(&input_clone2);
                        Ok(DidStart::Started(servers))
                    },
                ))
            }
            Ok(SystemStartArgs::InputOnly { input }) => {
                debug!("SystemStartArgs::InputOnly");

                let addr = ctx.address();
                let input_clone2 = input.clone();
                let jobs = setup_jobs(addr, input.clone());

                Box::pin(jobs.into_actor(self).map(
                    move |res: Result<SetupOk, anyhow::Error>, actor, _ctx| {
                        let res = res?;
                        debug!("✅ setup jobs completed");
                        let errored = ChildResult::first_server_error(&res.child_results);
                        if let Some(server_error) = errored {
                            debug!("errored: {:?}", errored);
                            return Err(StartupError::ServerError((*server_error).to_owned()));
                        }
                        actor.accept_watchables(&input_clone2);
                        Ok(DidStart::Started(res.servers))
                    },
                ))
            }
            Ok(SystemStartArgs::PathWithInvalidInput { path, input_error }) => {
                debug!("SystemStartArgs::PathWithInvalidInput");
                ctx.notify(MonitorInput {
                    path: path.clone(),
                    cwd: cwd.clone(),
                    input_ctx: InputCtx::default(),
                });
                self.publish_any_event(AnyEvent::Internal(InternalEvents::InputError(input_error)));
                let f = ready(Ok(DidStart::Started(Default::default()))).into_actor(self);
                Box::pin(f)
            }
            Ok(SystemStartArgs::RunOnly {
                input,
                named,
                run_mode: RunMode::Exec,
                top_level_run_mode,
            }) => {
                let addr = ctx.address();
                let jobs = run_jobs(addr, input.clone(), named, top_level_run_mode);
                Box::pin(jobs.into_actor(self).map(
                    move |res: Result<RunOk, anyhow::Error>, _actor, _ctx| match res {
                        Ok(_) => Ok(DidStart::WillExit),
                        Err(err) => Err(StartupError::Any(err.into())),
                    },
                ))
            }
            Ok(SystemStartArgs::RunOnly {
                input,
                named,
                run_mode: RunMode::Dry,
                top_level_run_mode,
            }) => {
                let addr = ctx.address();
                let jobs = print_jobs(addr, input.clone(), named, top_level_run_mode);
                Box::pin(jobs.into_actor(self).map(
                    move |res: Result<RunDryOk, anyhow::Error>, actor, _ctx| match res {
                        Ok(RunDryOk { tree, spec: _ }) => {
                            actor.publish_any_event(AnyEvent::Internal(TaskSpecDisplay { tree }));
                            Ok(DidStart::WillExit)
                        }
                        Err(err) => Err(StartupError::Any(err.into())),
                    },
                ))
            }
            Err(e) => {
                let f = ready(Err(StartupError::InputError(*e))).into_actor(self);
                Box::pin(f)
            }
        }
    }
}

#[derive(Debug, actix::Message)]
#[rtype(result = "Result<(GetActiveServersResponse, Vec<ChildResult>), ServerError>")]
pub struct OverrideInput {
    pub input: Input,
    pub original_event: AnyEvent,
}

impl actix::Handler<OverrideInput> for BsSystem {
    type Result =
        ResponseActFuture<Self, Result<(GetActiveServersResponse, Vec<ChildResult>), ServerError>>;

    fn handle(&mut self, msg: OverrideInput, ctx: &mut Self::Context) -> Self::Result {
        let input_clone = msg.input.clone();
        let start_ctx_clone = self
            .start_context
            .clone()
            .expect("If we get here, it's a big problem");
        // let ctx_clone = self.st
        let f = ctx
            .address()
            .send(ResolveServers { input: msg.input })
            .into_actor(self)
            .map(move |res, actor, _ctx| {
                debug!(" + did override input");
                let output = match res {
                    Ok(Ok(res)) => Ok(res),
                    Ok(Err(s_e)) => Err(s_e),
                    Err(err) => Err(ServerError::Unknown(err.to_string())),
                };
                actor.accept_watchables(&input_clone);
                actor.update_ctx(&input_clone, &start_ctx_clone);
                output
            });
        Box::pin(f)
    }
}

#[derive(actix::Message)]
#[rtype(result = "Result<(GetActiveServersResponse, Vec<ChildResult>), ServerError>")]
struct ResolveServers {
    input: Input,
}

impl actix::Handler<ResolveServers> for BsSystem {
    type Result = ResponseFuture<Result<(GetActiveServersResponse, Vec<ChildResult>), ServerError>>;

    #[tracing::instrument(skip_all, name = "Handler->ResolveServers->BsSystem")]
    fn handle(&mut self, msg: ResolveServers, _ctx: &mut Self::Context) -> Self::Result {
        let Some(servers_addr) = &self.servers_addr else {
            unreachable!("self.servers_addr cannot be absent?");
        };
        let external_event_sender = self.any_event_sender.as_ref().unwrap().clone();

        let addr = servers_addr.clone();

        let f = async move {
            debug!("will mark input as changed or new");
            let results = addr.send(InputChanged { input: msg.input }).await;

            let Ok(result_set) = results else {
                let e = results.unwrap_err();
                unreachable!("?1 {:?}", e);
            };

            debug!(
                "result_set from resolve servers {}",
                result_set.changes.len()
            );

            for (maybe_addr, x) in &result_set.changes {
                match x {
                    ChildResult::Stopped(id) => addr.do_send(ChildStopped {
                        identity: id.clone(),
                    }),
                    ChildResult::Created(c) if maybe_addr.is_some() => {
                        let child_handler = ChildHandler {
                            actor_address: maybe_addr.clone().expect("guarded above"),
                            identity: c.server_handler.identity.clone(),
                            socket_addr: c.server_handler.socket_addr,
                        };
                        addr.do_send(ChildCreatedInsert { child_handler })
                    }
                    ChildResult::Created(_c) => {
                        unreachable!("can't be created without")
                    }
                    ChildResult::Patched(p) if maybe_addr.is_some() => {
                        if let Some(child_actor) = maybe_addr {
                            child_actor.do_send(ClientConfigChange {
                                change_set: p.client_config_change_set.clone(),
                            });
                            child_actor.do_send(RoutesUpdated {
                                change_set: p.route_change_set.clone(),
                            })
                        } else {
                            tracing::error!("missing actor addr where it was needed")
                        }
                    }
                    ChildResult::Patched(p) => {
                        debug!("ChildResult::Patched {:?}", p);
                    }
                    ChildResult::PatchErr(e) => {
                        debug!("ChildResult::PatchErr {:?}", e);
                    }
                    ChildResult::CreateErr(e) => {
                        debug!("ChildResult::CreateErr {:?}", e);
                    }
                }
            }

            let res = result_set
                .changes
                .into_iter()
                .map(|(_, child_result)| child_result)
                .collect::<Vec<_>>();

            match addr.send(GetActiveServers).await {
                Ok(resp) => {
                    Arbiter::current().spawn({
                        let evt = InternalEvents::ServersChanged {
                            server_resp: resp.clone(),
                            child_results: res.clone(),
                        };
                        debug!("will emit {:?}", evt);
                        async move {
                            match external_event_sender.send(AnyEvent::Internal(evt)).await {
                                Ok(_) => {}
                                Err(e) => debug!(?e),
                            };
                        }
                    });
                    Ok((resp, res))
                }
                Err(e) => Err(ServerError::Unknown(e.to_string())),
            }
        };

        Box::pin(f)
    }
}

#[derive(actix::Message)]
#[rtype(result = "Result<TaskReportAndTree, InitialTaskError>")]
struct ResolveInitialTasks {
    input: Input,
}

impl actix::Handler<ResolveInitialTasks> for BsSystem {
    type Result = ResponseFuture<Result<TaskReportAndTree, InitialTaskError>>;

    #[tracing::instrument(skip_all, name = "Handler->ResolveInitialTasks->BsSystem")]
    fn handle(&mut self, msg: ResolveInitialTasks, ctx: &mut Self::Context) -> Self::Result {
        let addr = ctx.address();
        let (next, rx) = self.before(&msg.input, addr);
        ctx.notify(next);

        Box::pin(async move {
            match rx.await {
                Ok(TaskReportAndTree { report, tree }) if report.is_ok() => {
                    Ok(TaskReportAndTree { report, tree })
                }
                Ok(TaskReportAndTree { .. }) => Err(InitialTaskError::FailedReport),
                Err(_) => Err(InitialTaskError::FailedUnknown),
            }
        })
    }
}

#[derive(actix::Message)]
#[rtype(result = "Result<GetActiveServersResponse, ServerError>")]
struct ReadActiveServers;

impl actix::Handler<ReadActiveServers> for BsSystem {
    type Result = ResponseFuture<Result<GetActiveServersResponse, ServerError>>;

    fn handle(&mut self, _msg: ReadActiveServers, _ctx: &mut Self::Context) -> Self::Result {
        let Some(addr) = self.servers_addr.as_ref() else {
            unreachable!("This cannot occur?");
        };
        let cloned_address = addr.clone();

        Box::pin(async move {
            match cloned_address.send(GetActiveServers).await {
                Ok(resp) => Ok(resp),
                Err(e) => Err(ServerError::Unknown(e.to_string())),
            }
        })
    }
}
