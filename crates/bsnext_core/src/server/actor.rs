use crate::dto::ClientEvent;
use crate::server::handler_stop::Stop;
use crate::server::signals::ServerSignals;
use crate::server::state::ServerState;
use actix::{ActorContext, AsyncContext, Running};
use axum_server::Handle;
use bsnext_input::route_manifest::RoutesManifest;
use bsnext_input::server_config::ServerConfig;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::oneshot::Sender;
use tokio::sync::{broadcast, oneshot};
use tracing::{span, Level};

pub struct ServerActor {
    pub config: ServerConfig,
    pub routes_manifest: RoutesManifest,
    pub signals: Option<ServerSignals>,
    pub app_state: Option<Arc<ServerState>>,
    pub addr: Option<SocketAddr>,
}

impl ServerActor {
    pub fn new_from_config(config: ServerConfig) -> Self {
        let routes_manifest = RoutesManifest::new(&config.routes);
        Self {
            config,
            signals: None,
            app_state: None,
            addr: None,
            routes_manifest,
        }
    }
    pub fn install_signals(&mut self) -> (Sender<()>, Handle, broadcast::Sender<ClientEvent>) {
        // todo: make this an enum for more messages
        let (client_sender, client_receiver) = broadcast::channel::<ClientEvent>(5);
        let (shutdown_complete, shutdown_complete_receiver) = oneshot::channel();
        let axum_server_handle = Handle::new();
        let axum_server_handle_clone = axum_server_handle.clone();

        self.signals = Some(ServerSignals {
            complete_mdg_receiver: Some(shutdown_complete_receiver),
            axum_server_handle: Some(axum_server_handle),
            client_sender: Some(client_sender.clone()),
            client_receiver: Some(client_receiver),
        });

        (shutdown_complete, axum_server_handle_clone, client_sender)
    }
}

impl actix::Actor for ServerActor {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(actor.name = "ServerActor", actor.lifecyle = "started");
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        let span = span!(
            Level::TRACE,
            "stopping",
            actor.name = "ServerActor",
            actor.lifecyle = "stopping",
            identity = ?self.config.identity
        );
        let _guard = span.enter();
        let addr = ctx.address();
        let s = ctx.state();

        // this handles crashes. If the server actor crashes we want to ensure we're
        // cleaning up any server we've started. So in a crash we return 'Running::Continue'
        // and send ourselves the `Stop2` message (which will cleanly close the server)
        // `Stop2` will then eventually also try to close this actor, at which point we'll allow it
        if s.alive() {
            tracing::trace!("actor was ❤️, sending `Stop2` to SELF",);
            addr.do_send(Stop);
            Running::Continue
        } else {
            tracing::trace!("Server actor was already 💀");
            Running::Stop
        }
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(
            actor.name = "ServerActor",
            actor.lifecyle = "stopped",
            identity = ?self.config.identity
        );
    }
}
