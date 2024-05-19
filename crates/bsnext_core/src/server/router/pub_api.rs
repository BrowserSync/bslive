use crate::dto::{RouteDTO, ServerDesc};
use crate::server::state::ServerState;
use crate::servers_supervisor::get_servers_handler::GetServersMessage;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use http::{StatusCode, Uri};
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
        .with_state(state.clone())
}
