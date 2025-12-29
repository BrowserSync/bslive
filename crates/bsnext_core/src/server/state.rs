use crate::runtime_ctx::RuntimeCtx;
use crate::servers_supervisor::get_servers_handler::{GetActiveServers, IncomingEvents};
use actix::Recipient;
use axum::Router;
use bsnext_dto::ClientEvent;
use bsnext_input::client_config::ClientConfig;
use bsnext_input::route::Route;
use std::fmt::Formatter;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use tokio::sync::{broadcast, RwLock};

#[derive(Clone)]
pub struct ServerState {
    pub routes: Arc<RwLock<Vec<Route>>>,
    pub runtime_ctx: RuntimeCtx,
    pub raw_router: Arc<RwLock<Router>>,
    pub client_config: Arc<RwLock<ClientConfig>>,
    pub socket_addr: Arc<Mutex<Option<SocketAddr>>>,
    pub id: u64,
    pub parent: Option<Recipient<GetActiveServers>>,
    pub evt_receiver: Option<Recipient<IncomingEvents>>,
    pub client_sender: Arc<broadcast::Sender<ClientEvent>>,
}

impl std::fmt::Debug for ServerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("routes", &"Arc<RwLock<Vec<Route>>>")
            .finish()
    }
}
