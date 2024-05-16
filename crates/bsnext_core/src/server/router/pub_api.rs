use crate::dto::{RouteDTO, ServerDesc};
use crate::server::state::ServerState;
use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use http::Uri;
use std::sync::Arc;

async fn api_handler(State(app): State<Arc<ServerState>>, _uri: Uri) -> impl IntoResponse {
    let routes = app.routes.read().await;
    let dto = ServerDesc {
        routes: routes.iter().map(|r| RouteDTO::from(r)).collect(),
    };
    Json(dto)
}

pub fn pub_api(state: Arc<ServerState>) -> Router<Arc<ServerState>> {
    Router::new()
        .route("/servers", get(api_handler))
        .with_state(state.clone())
}
