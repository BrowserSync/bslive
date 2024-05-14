use crate::dto::ClientEvent;
use axum_server::Handle;
use tokio::sync::broadcast;
use tokio::sync::oneshot::Receiver;

pub struct ServerSignals {
    pub complete_mdg_receiver: Option<Receiver<()>>,
    pub axum_server_handle: Option<Handle>,
    pub client_sender: Option<broadcast::Sender<ClientEvent>>,
    pub client_receiver: Option<broadcast::Receiver<ClientEvent>>,
}
