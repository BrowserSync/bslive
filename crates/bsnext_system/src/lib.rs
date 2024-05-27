use crate::monitor::{
    to_route_watchables, to_server_watchables, AnyWatchable, Monitor, MonitorInput,
};
use actix::{Actor, Addr, AsyncContext, Handler, Running};

use bsnext_input::Input;
use std::collections::HashMap;

use actix_rt::Arbiter;
use bsnext_dto::{ExternalEvents, ServersStarted};
use std::path::PathBuf;

use bsnext_example::Example;

use crate::startup::{DidStart, StartupContext, StartupError, StartupResult, SystemStart};
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_core::servers_supervisor::get_servers_handler::GetServersMessage;
use bsnext_core::servers_supervisor::input_changed_handler::InputChanged;

use bsnext_fs::actor::FsWatcher;

use crate::monitor_any_watchables::MonitorAnyWatchables;
use start_kind::StartKind;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

pub mod args;
mod monitor;
mod monitor_any_watchables;
pub mod start_kind;
pub mod startup;

pub struct BsSystem {
    self_addr: Option<Addr<BsSystem>>,
    servers_addr: Option<Addr<ServersSupervisor>>,
    events_sender: Option<Sender<ExternalEvents>>,
    input_monitors: Vec<Addr<FsWatcher>>,
    any_monitors: HashMap<AnyWatchable, Monitor>,
    cwd: Option<PathBuf>,
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
            events_sender: None,
            input_monitors: vec![],
            any_monitors: Default::default(),
            cwd: None,
        }
    }

    fn accept_input(&mut self, input: &Input) {
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

        self_address.do_send(MonitorAnyWatchables {
            watchables: routes,
            cwd: cwd.clone(),
        });
    }

    fn inform_servers(&mut self, input: Input) {
        let Some(servers_addr) = &self.servers_addr else {
            unreachable!("self.servers_addr cannot exist?");
        };
        Arbiter::current().spawn({
            let addr = servers_addr.clone();
            let events_sender = self.events_sender.as_ref().unwrap().clone();
            async move {
                let results = addr.send(InputChanged { input }).await;
                let Ok(changeset) = results else {
                    unreachable!("?1")
                };
                let servers = addr.send(GetServersMessage).await;
                let Ok(servers_resp) = servers else {
                    unreachable!("?2")
                };

                let output = ExternalEvents::ServersStarted(ServersStarted {
                    servers_resp,
                    changeset,
                });
                match events_sender.send(output).await {
                    Ok(_) => tracing::trace!("Ok"),
                    Err(_) => tracing::trace!("Err"),
                };
            }
        });
    }

    fn ext_evt(&mut self, evt: ExternalEvents) {
        if let Some(sender) = &self.events_sender {
            Arbiter::current().spawn({
                let events_sender = sender.clone();
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
        self.events_sender = None;
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
    pub events_sender: Sender<ExternalEvents>,
    pub startup_oneshot_sender: oneshot::Sender<StartupResult>,
}

impl Handler<Start> for BsSystem {
    type Result = ();

    fn handle(&mut self, msg: Start, ctx: &mut Self::Context) -> Self::Result {
        self.events_sender = Some(msg.events_sender.clone());
        self.cwd = msg.cwd;

        let Some(cwd) = &self.cwd else {
            unreachable!("?")
        };

        let servers = ServersSupervisor::new(msg.ack);
        // store the servers addr for later
        self.servers_addr = Some(servers.start());

        let start_context = StartupContext::from_cwd(self.cwd.as_ref());

        match msg.kind.input(&start_context) {
            Ok((input, Some(path))) => {
                ctx.address().do_send(MonitorInput {
                    path: path.clone(),
                    cwd: cwd.clone(),
                });

                self.accept_input(&input);
                self.inform_servers(input);
            }
            Ok((input, None)) => {
                self.accept_input(&input);
                self.inform_servers(input);
            }
            Err(e) => {
                msg.startup_oneshot_sender
                    .send(Err(StartupError::InputError(e)))
                    .expect("oneshot must succeed");
                return;
            }
        }

        msg.startup_oneshot_sender
            .send(Ok(DidStart::Started))
            .expect("oneshot started must succeed")
    }
}
