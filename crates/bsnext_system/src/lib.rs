use crate::monitor::{
    to_route_watchables, to_server_watchables, AnyWatchable, Monitor, MonitorInput,
};
use actix::{Actor, Addr, AsyncContext, Handler, Running};

use bsnext_input::{Input, InputCtx};
use std::collections::HashMap;

use actix_rt::Arbiter;
use bsnext_dto::ExternalEventsDTO;
use std::path::PathBuf;
use std::sync::Arc;

use bsnext_example::Example;

use bsnext_core::servers_supervisor::actor::{ChildHandler, ChildStopped, ServersSupervisor};
use bsnext_core::servers_supervisor::input_changed_handler::InputChanged;

use bsnext_fs::actor::FsWatcher;

use crate::monitor_any_watchables::MonitorAnyWatchables;
use bsnext_core::server::handler_client_config::ClientConfigChange;
use bsnext_core::server::handler_routes_updated::RoutesUpdated;
use bsnext_core::servers_supervisor::get_servers_handler::GetServersMessage;
use bsnext_core::servers_supervisor::start_handler::ChildCreatedInsert;
use bsnext_dto::internal::{AnyEvent, ChildResult, InternalEvents};
use bsnext_input::startup::{
    DidStart, StartupContext, StartupError, StartupResult, SystemStart, SystemStartArgs,
};
use start_kind::StartKind;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tracing::{debug_span, Instrument};

pub mod args;
pub mod cli;
pub mod input_fs;
pub mod monitor;
mod monitor_any_watchables;
pub mod start_kind;

pub struct BsSystem {
    self_addr: Option<Addr<BsSystem>>,
    servers_addr: Option<Addr<ServersSupervisor>>,
    any_event_sender: Option<Sender<AnyEvent>>,
    input_monitors: Option<InputMonitor>,
    any_monitors: HashMap<AnyWatchable, Monitor>,
    cwd: Option<PathBuf>,
}

#[derive(Debug, Clone)]
pub struct InputMonitor {
    pub addr: Addr<FsWatcher>,
    pub ctx: InputCtx,
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

impl Handler<StopSystem> for BsSystem {
    type Result = ();

    fn handle(&mut self, _msg: StopSystem, _ctx: &mut Self::Context) -> Self::Result {
        todo!("can the system as a whole be stopped?")
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
            cwd: None,
        }
    }

    fn accept_watchables(&mut self, input: &Input) {
        let span = debug_span!("accept_input");
        let s = Arc::new(span);
        let c = s.clone();
        let _c2 = s.clone();
        let _g = s.enter();
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
            .map(|r| AnyWatchable::Route(r.to_owned()))
            .collect::<Vec<_>>();

        let servers = server_watchables
            .iter()
            .map(|w| AnyWatchable::Server(w.to_owned()))
            .collect::<Vec<_>>();

        all_watchables.extend(servers);
        let cwd = cwd.clone();
        let addr = self_address.clone();

        Arbiter::current().spawn(
            async move {
                match addr
                    .send(MonitorAnyWatchables {
                        watchables: all_watchables,
                        cwd,
                        span: c,
                    })
                    .await
                {
                    Ok(_) => tracing::info!("sent"),
                    Err(e) => tracing::error!(%e),
                };
            }
            .in_current_span(),
        );
    }

    fn resolve_servers(&mut self, input: Input) {
        let Some(servers_addr) = &self.servers_addr else {
            unreachable!("self.servers_addr cannot be absent?");
        };
        Arbiter::current().spawn({
            let addr = servers_addr.clone();
            let external_event_sender = self.any_event_sender.as_ref().unwrap().clone();
            let inner = debug_span!("inform_servers");
            let _g = inner.enter();

            async move {
                let results = addr.send(InputChanged { input }).await;

                let Ok(result_set) = results else {
                    let e = results.unwrap_err();
                    unreachable!("?1 {:?}", e);
                };

                for (maybe_addr, x) in &result_set {
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
                            let inner = debug_span!("patching...");
                            let _g = inner.enter();
                            if let Some(child_actor) = maybe_addr {
                                child_actor.do_send(ClientConfigChange {
                                    change_set: p.client_config_change_set.clone(),
                                });
                                child_actor.do_send(RoutesUpdated {
                                    change_set: p.route_change_set.clone(),
                                    span: Arc::new(inner.clone()),
                                })
                            } else {
                                tracing::error!("missing actor addr where it was needed")
                            }
                        }
                        ChildResult::Patched(_p) => {
                            todo!("not implemented yet")
                        }
                        ChildResult::PatchErr(_) => {}
                        ChildResult::CreateErr(_) => {}
                    }
                }

                // dbg!(result_set);
                let servers = addr.send(GetServersMessage).await;
                let Ok(servers_resp) = servers else {
                    unreachable!("?2")
                };

                let res = result_set
                    .into_iter()
                    .map(|(_, child_result)| child_result)
                    .collect();

                let evt = InternalEvents::ServersChanged {
                    server_resp: servers_resp,
                    child_results: res,
                };

                match external_event_sender.send(AnyEvent::Internal(evt)).await {
                    Ok(_) => tracing::trace!("Ok"),
                    Err(_) => tracing::trace!("Err"),
                };
            }
            .in_current_span()
        });
    }

    #[tracing::instrument(skip(self))]
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
                .in_current_span()
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

#[derive(actix::Message)]
#[rtype(result = "StartupResult")]
pub struct Start {
    pub kind: StartKind,
    pub cwd: Option<PathBuf>,
    pub ack: oneshot::Sender<()>,
    pub events_sender: Sender<AnyEvent>,
}

impl Handler<Start> for BsSystem {
    type Result = StartupResult;

    fn handle(&mut self, msg: Start, ctx: &mut Self::Context) -> Self::Result {
        self.any_event_sender = Some(msg.events_sender.clone());
        self.cwd = msg.cwd;

        let Some(cwd) = &self.cwd else {
            unreachable!("?")
        };

        tracing::debug!("{:?}", self.cwd);

        let servers = ServersSupervisor::new(msg.ack);
        // store the servers addr for later
        self.servers_addr = Some(servers.start());

        let start_context = StartupContext::from_cwd(self.cwd.as_ref());

        tracing::debug!(?start_context);

        match msg.kind.input(&start_context) {
            Ok(SystemStartArgs::PathWithInput { path, input }) => {
                tracing::debug!("PathWithInput");
                let ids = input
                    .servers
                    .iter()
                    .map(|x| x.identity.clone())
                    .collect::<Vec<_>>();
                let input_ctx = InputCtx::new(&ids);
                ctx.notify(MonitorInput {
                    path: path.clone(),
                    cwd: cwd.clone(),
                    ctx: input_ctx,
                });

                self.accept_watchables(&input);
                self.resolve_servers(input);
                Ok(DidStart::Started)
            }
            Ok(SystemStartArgs::InputOnly { input }) => {
                tracing::debug!("InputOnly");
                self.accept_watchables(&input);
                self.resolve_servers(input);
                Ok(DidStart::Started)
            }
            Ok(SystemStartArgs::PathWithInvalidInput { path, input_error }) => {
                tracing::debug!("PathWithInvalidInput");
                ctx.notify(MonitorInput {
                    path: path.clone(),
                    cwd: cwd.clone(),
                    ctx: InputCtx::default(),
                });
                self.publish_any_event(AnyEvent::Internal(InternalEvents::InputError(input_error)));
                Ok(DidStart::Started)
            }
            Err(e) => Err(StartupError::InputError(*e)),
        }
    }
}
