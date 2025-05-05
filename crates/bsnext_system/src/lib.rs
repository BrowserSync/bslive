use crate::any_monitor::{AnyMonitor, PathWatchable};
use actix::{
    Actor, ActorContext, Addr, AsyncContext, Handler, ResponseActFuture, ResponseFuture, Running,
    WrapFuture,
};

use actix::ActorFutureExt;
use actix_rt::Arbiter;
use bsnext_core::servers_supervisor::actor::{ChildHandler, ChildStopped, ServersSupervisor};
use bsnext_core::servers_supervisor::input_changed_handler::InputChanged;
use bsnext_input::{Input, InputCtx};
use std::collections::HashMap;
use std::path::PathBuf;

use crate::monitor_any_watchables::MonitorPathWatchables;
use crate::runner::Runner;
use bsnext_core::server::handler_client_config::ClientConfigChange;
use bsnext_core::server::handler_routes_updated::RoutesUpdated;
use bsnext_core::servers_supervisor::get_servers_handler::GetActiveServers;
use bsnext_core::servers_supervisor::start_handler::ChildCreatedInsert;
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal::{AnyEvent, ChildNotCreated, ChildResult, InternalEvents, ServerError};
use bsnext_dto::{ActiveServer, DidStart, GetActiveServersResponse, StartupError};
use bsnext_fs::{FsEvent, FsEventContext};
use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
use input_monitor::{InputMonitor, MonitorInput};
use route_watchable::to_route_watchables;
use server_watchable::to_server_watchables;
use start::start_kind::StartKind;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

pub mod any_monitor;
pub mod archy;
pub mod args;
pub mod cli;
pub mod export;
mod ext_event_sender;
mod handle_fs_event;
pub mod input_fs;
mod input_monitor;
mod input_watchable;
mod monitor_any_watchables;
mod path_monitor;
mod route_watchable;
pub mod runner;
pub mod server_watchable;
pub mod start;
pub mod task;
pub mod task_group;
pub mod task_group_runner;
pub mod tasks;

#[derive(Debug)]
pub(crate) struct BsSystem {
    self_addr: Option<Addr<BsSystem>>,
    servers_addr: Option<Addr<ServersSupervisor>>,
    any_event_sender: Option<Sender<AnyEvent>>,
    input_monitors: Option<InputMonitor>,
    any_monitors: HashMap<PathWatchable, AnyMonitor>,
    tasks: HashMap<FsEventContext, Runner>,
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
        self.sys_address.do_send(evt)
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
            tasks: Default::default(),
            cwd: None,
            start_context: None,
        }
    }

    fn accept_watchables(&mut self, input: &Input) {
        let route_watchables = to_route_watchables(input);
        let server_watchables = to_server_watchables(input);

        tracing::debug!(
            "accepting {} route watchables, and {} server watchables",
            route_watchables.len(),
            server_watchables.len()
        );

        let Some(self_address) = &self.self_addr else {
            unreachable!("?")
        };

        let Some(cwd) = &self.cwd else {
            unreachable!("can this occur?")
        };

        // todo: clean up this merging
        let mut all_watchables = route_watchables
            .iter()
            .map(|r| PathWatchable::Route(r.to_owned()))
            .collect::<Vec<_>>();

        let servers = server_watchables
            .iter()
            .map(|w| PathWatchable::Server(w.to_owned()))
            .collect::<Vec<_>>();

        all_watchables.extend(servers);
        let cwd = cwd.clone();
        let addr = self_address.clone();
        let msg = MonitorPathWatchables {
            watchables: all_watchables,
            cwd,
        };

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
        tracing::debug!(?evt);

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

#[derive(actix::Message)]
#[rtype(result = "Result<DidStart, StartupError>")]
pub struct Start {
    pub kind: StartKind,
    pub cwd: PathBuf,
    pub ack: oneshot::Sender<()>,
    pub events_sender: Sender<AnyEvent>,
}

impl Handler<Start> for BsSystem {
    type Result = ResponseActFuture<Self, Result<DidStart, StartupError>>;

    fn handle(&mut self, msg: Start, ctx: &mut Self::Context) -> Self::Result {
        self.any_event_sender = Some(msg.events_sender.clone());
        self.cwd = Some(msg.cwd);

        tracing::debug!("self.cwd {:?}", self.cwd);

        let Some(cwd) = &self.cwd else {
            unreachable!("?")
        };

        let servers = ServersSupervisor::new(msg.ack);
        // store the servers addr for later
        self.servers_addr = Some(servers.start());

        let start_context = StartupContext::from_cwd(self.cwd.as_ref());
        self.start_context = Some(start_context.clone());

        tracing::debug!(?start_context);

        match msg.kind.input(&start_context) {
            Ok(SystemStartArgs::PathWithInput { path, input }) => {
                tracing::debug!("SystemStartArgs::PathWithInput");
                let ids = input
                    .servers
                    .iter()
                    .map(|x| x.identity.clone())
                    .collect::<Vec<_>>();

                let input_ctx = InputCtx::new(&ids, None, &start_context, Some(&path));
                let input_clone = input.clone();

                let f = ctx
                    .address()
                    .send(ResolveServers { input })
                    .into_actor(self)
                    .map(move |res, actor, ctx| match res {
                        Ok(Ok((resp, _))) => {
                            ctx.notify(MonitorInput {
                                path: path.clone(),
                                cwd: actor.cwd.clone().unwrap(),
                                input_ctx,
                            });
                            // todo: where to better sequence these side-effects
                            actor.accept_watchables(&input_clone);
                            Ok(DidStart::Started(resp))
                        }
                        Ok(Err(e)) => Err(StartupError::Other(e.to_string())),
                        Err(e) => Err(StartupError::Other(e.to_string())),
                    });

                Box::pin(f)
            }
            Ok(SystemStartArgs::InputOnly { input }) => {
                tracing::debug!("SystemStartArgs::InputOnly");

                let input_clone = input.clone();
                let f = ctx
                    .address()
                    .send(ResolveServers { input })
                    .into_actor(self)
                    .map(move |res, actor, _ctx| match res {
                        Ok(Ok((resp, child_results))) => {
                            let errored = child_results
                                .iter()
                                .find(|x| matches!(x, ChildResult::CreateErr(..)));
                            if let Some(ChildResult::CreateErr(ChildNotCreated {
                                server_error,
                                ..
                            })) = errored
                            {
                                tracing::debug!("errored: {:?}", errored);
                                Err(StartupError::ServerError((*server_error).to_owned()))
                            } else {
                                actor.accept_watchables(&input_clone);
                                Ok(DidStart::Started(resp))
                            }
                        }
                        Ok(Err(e)) => {
                            tracing::debug!(?e, "server error");
                            Err(StartupError::Other(e.to_string()))
                        }
                        Err(e) => {
                            tracing::debug!(?e, "other error");
                            Err(StartupError::Other(e.to_string()))
                        }
                    });
                Box::pin(f)
            }
            Ok(SystemStartArgs::PathWithInvalidInput { path, input_error }) => {
                tracing::debug!("SystemStartArgs::PathWithInvalidInput");
                ctx.notify(MonitorInput {
                    path: path.clone(),
                    cwd: cwd.clone(),
                    input_ctx: InputCtx::default(),
                });
                self.publish_any_event(AnyEvent::Internal(InternalEvents::InputError(input_error)));
                let f = async move { Ok(DidStart::Started(Default::default())) }.into_actor(self);
                Box::pin(f)
            }
            Err(e) => {
                let f = async move { Err(StartupError::InputError(*e)) }.into_actor(self);
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
                tracing::debug!(" + did override input");
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

    fn handle(&mut self, msg: ResolveServers, _ctx: &mut Self::Context) -> Self::Result {
        let Some(servers_addr) = &self.servers_addr else {
            unreachable!("self.servers_addr cannot be absent?");
        };
        let external_event_sender = self.any_event_sender.as_ref().unwrap().clone();

        let addr = servers_addr.clone();

        let f = async move {
            tracing::debug!("will mark input as changed");
            let results = addr.send(InputChanged { input: msg.input }).await;

            let Ok(result_set) = results else {
                let e = results.unwrap_err();
                unreachable!("?1 {:?}", e);
            };

            tracing::debug!(
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
                        tracing::debug!("ChildResult::Patched {:?}", p);
                    }
                    ChildResult::PatchErr(e) => {
                        tracing::debug!("ChildResult::PatchErr {:?}", e);
                    }
                    ChildResult::CreateErr(e) => {
                        tracing::debug!("ChildResult::CreateErr {:?}", e);
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
                        tracing::debug!("will emit {:?}", evt);
                        async move {
                            match external_event_sender.send(AnyEvent::Internal(evt)).await {
                                Ok(_) => {}
                                Err(e) => tracing::debug!(?e),
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
