use crate::server::state::ServerState;
use crate::servers_supervisor::file_changed_handler::FilesChanged;
use crate::servers_supervisor::get_servers_handler::{GetServersMessage, IncomingEvents};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::{Json, Router};
use bsnext_dto::{ChangeDTO, ClientEvent, RouteDTO, ServerDesc};
use bsnext_fs::FsEventContext;
use http::{StatusCode, Uri};
use serde_json::json;
use std::path::PathBuf;
use std::sync::Arc;

async fn all_servers_handler(State(app): State<Arc<ServerState>>, _uri: Uri) -> impl IntoResponse {
    match &app.parent {
        None => unreachable!("cannot get here"),
        Some(parent) => match parent.send(GetServersMessage).await {
            Ok(servers) => Json(servers).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Could not fetch servers: {}", err),
            )
                .into_response(),
        },
    }
}

async fn get_current_server(State(app): State<Arc<ServerState>>, _uri: Uri) -> impl IntoResponse {
    let routes = app.routes.read().await;
    let dto = ServerDesc {
        routes: routes.iter().map(RouteDTO::from).collect(),
        id: app.id.to_string(),
    };
    Json(dto)
}

pub fn pub_api(state: Arc<ServerState>) -> Router<Arc<ServerState>> {
    Router::new()
        .route("/servers", get(all_servers_handler))
        .route("/me", get(get_current_server))
        .route("/events", post(post_events))
        .with_state(state.clone())
}

async fn post_events(
    State(app): State<Arc<ServerState>>,
    Json(payload): Json<ClientEvent>,
) -> impl IntoResponse {
    tracing::trace!("Got post event: {:?}", payload);
    match &app.evt_receiver {
        None => unreachable!("should be unreachable?"),
        Some(recv) => {
            let paths = match payload {
                ClientEvent::Change(ChangeDTO::Fs { path, .. }) => {
                    vec![PathBuf::from(path)]
                }
                ClientEvent::Change(ChangeDTO::FsMany(changes)) => changes
                    .into_iter()
                    .map(|x| match x {
                        ChangeDTO::Fs { path, .. } => PathBuf::from(path),
                        ChangeDTO::FsMany(..) => todo!("recursion..."),
                    })
                    .collect::<Vec<_>>(),
                ClientEvent::Config(_) => {
                    todo!("handle ClientEvent::Config  in incoming event handler...")
                }
            };
            match recv
                .send(IncomingEvents::FilesChanged(FilesChanged {
                    paths,
                    ctx: FsEventContext { id: app.id },
                }))
                .await
            {
                Ok(_) => {}
                Err(err) => tracing::error!(?err),
            };
        }
    }
    Json(json!({"ok": true})).into_response()
}
