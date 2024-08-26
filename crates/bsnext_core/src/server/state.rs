use crate::servers_supervisor::get_servers_handler::{GetServersMessage, IncomingEvents};
use actix::Recipient;
use axum::Router;
use bsnext_dto::ClientEvent;
use bsnext_input::route::Route;
use std::fmt::Formatter;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Clone)]
pub struct ServerState {
    pub routes: Arc<RwLock<Vec<Route>>>,
    pub router: Arc<RwLock<Router>>,
    pub id: u64,
    pub parent: Option<Recipient<GetServersMessage>>,
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
