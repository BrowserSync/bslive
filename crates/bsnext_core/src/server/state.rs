use crate::dto::ClientEvent;
use bsnext_input::route::Route;
use std::fmt::Formatter;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

#[derive(Clone)]
pub struct ServerState {
    pub routes: Arc<RwLock<Vec<Route>>>,
    pub client_sender: Arc<broadcast::Sender<ClientEvent>>,
}

impl std::fmt::Debug for ServerState {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState")
            .field("routes", &"Arc<RwLock<Vec<Route>>>")
            .finish()
    }
}
