use crate::monitor::{
    to_route_watchables, to_server_watchables, AnyWatchable, Monitor, MonitorInput,
};
use actix::{Actor, Addr, AsyncContext, Handler, Running};

use bsnext_input::Input;
use std::collections::HashMap;

use actix_rt::Arbiter;
use bsnext_dto::{ExternalEvents, ServersStarted};
use std::path::PathBuf;
use std::sync::Arc;

use bsnext_example::Example;

use crate::startup::{DidStart, StartupContext, StartupResult, SystemStart, SystemStartArgs};
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_core::servers_supervisor::get_servers_handler::GetServersMessage;
use bsnext_core::servers_supervisor::input_changed_handler::InputChanged;

use bsnext_fs::actor::FsWatcher;

use crate::monitor_any_watchables::MonitorAnyWatchables;
use start_kind::StartKind;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tracing::{debug_span, Instrument};

pub mod args;
pub mod cli;
mod monitor;
mod monitor_any_watchables;
pub mod start_kind;
pub mod startup;

pub struct BsSystem {
    self_addr: Option<Addr<BsSystem>>,
    servers_addr: Option<Addr<ServersSupervisor>>,
    external_event_sender: Option<Sender<EventWithSpan>>,
    input_monitors: Vec<Addr<FsWatcher>>,
    any_monitors: HashMap<AnyWatchable, Monitor>,
    cwd: Option<PathBuf>,
}

#[derive(Debug)]
pub struct EventWithSpan {
    pub evt: ExternalEvents,
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
            external_event_sender: None,
            input_monitors: vec![],
            any_monitors: Default::default(),
            cwd: None,
        }
    }

    fn accept_input(&mut self, input: &Input) {
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
        let mut routes = route_watchables
            .iter()
            .map(|r| AnyWatchable::Route(r.to_owned()))
            .collect::<Vec<_>>();

        let servers = server_watchables
            .iter()
            .map(|w| AnyWatchable::Server(w.to_owned()))
            .collect::<Vec<_>>();

        routes.extend(servers);
        let cwd = cwd.clone();
        let addr = self_address.clone();

        Arbiter::current().spawn(
            async move {
                match addr
                    .send(MonitorAnyWatchables {
                        watchables: routes,
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

    fn inform_servers(&mut self, input: Input) {
        let Some(servers_addr) = &self.servers_addr else {
            unreachable!("self.servers_addr cannot exist?");
        };
        Arbiter::current().spawn({
            let addr = servers_addr.clone();
            let external_event_sender = self.external_event_sender.as_ref().unwrap().clone();
            let inner = debug_span!("inform_servers");
            let _g = inner.enter();

            async move {
                let results = addr.send(InputChanged { input }).await;
                let Ok(changeset) = results else {
                    unreachable!("?1")
                };
                let servers = addr.send(GetServersMessage).await;
                let Ok(servers_resp) = servers else {
                    unreachable!("?2")
                };

                let evt = ExternalEvents::ServersStarted(ServersStarted {
                    servers_resp,
                    changeset,
                });

                let out = EventWithSpan { evt };

                match external_event_sender.send(out).await {
                    Ok(_) => tracing::trace!("Ok"),
                    Err(_) => tracing::trace!("Err"),
                };
            }
            .in_current_span()
        });
    }

    #[tracing::instrument(skip(self))]
    fn publish_external_event(&mut self, evt: ExternalEvents) {
        tracing::debug!(?evt);
        let outgoing = EventWithSpan { evt };
        if let Some(external_event_sender) = &self.external_event_sender {
            Arbiter::current().spawn({
                let events_sender = external_event_sender.clone();
                async move {
                    match events_sender.send(outgoing).await {
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
        self.external_event_sender = None;
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct StopSystem;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Start {
    pub kind: StartKind,
    pub cwd: Option<PathBuf>,
    pub ack: oneshot::Sender<()>,
    pub events_sender: Sender<EventWithSpan>,
    pub startup_oneshot_sender: oneshot::Sender<StartupResult>,
}

impl Handler<Start> for BsSystem {
    type Result = ();

    fn handle(&mut self, msg: Start, ctx: &mut Self::Context) -> Self::Result {
        self.external_event_sender = Some(msg.events_sender.clone());
        self.cwd = msg.cwd;

        let Some(cwd) = &self.cwd else {
            unreachable!("?")
        };

        let servers = ServersSupervisor::new(msg.ack);
        // store the servers addr for later
        self.servers_addr = Some(servers.start());

        let start_context = StartupContext::from_cwd(self.cwd.as_ref());

        match msg.kind.input(&start_context) {
            Ok(SystemStartArgs::PathWithInput { path, input }) => {
                ctx.address().do_send(MonitorInput {
                    path: path.clone(),
                    cwd: cwd.clone(),
                });

                self.accept_input(&input);
                self.inform_servers(input);
            }
            Ok(SystemStartArgs::InputOnly { input }) => {
                self.accept_input(&input);
                self.inform_servers(input);
            }
            Ok(SystemStartArgs::PathWithInvalidInput { path, input_error }) => {
                tracing::debug!("PathWithInvalidInput");
                ctx.address().do_send(MonitorInput {
                    path: path.clone(),
                    cwd: cwd.clone(),
                });
                self.publish_external_event(ExternalEvents::InputError(input_error.into()));
            }
            Err(e) => {
                tracing::error!(?e);
            }
        }

        msg.startup_oneshot_sender
            .send(Ok(DidStart::Started))
            .expect("oneshot started must succeed")
    }
}
