use crate::server::state::ServerState;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::Router;
use bsnext_client::{UI_CSS, UI_JS};
use http::header::CONTENT_TYPE;
use http::HeaderValue;
use mime_guess::mime;
use std::sync::Arc;

pub fn pub_ui_assets(_: Arc<ServerState>) -> Router<Arc<ServerState>> {
    Router::new()
        .route(
            "/dist/index.js",
            get(|| async {
                (
                    [(
                        CONTENT_TYPE,
                        HeaderValue::from_static(mime::APPLICATION_JAVASCRIPT.as_ref()),
                    )],
                    UI_JS,
                )
                    .into_response()
            }),
        )
        .route(
            "/dist/index.css",
            get(|| async {
                (
                    [(
                        CONTENT_TYPE,
                        HeaderValue::from_static(mime::TEXT_CSS.as_ref()),
                    )],
                    UI_CSS,
                )
                    .into_response()
            }),
        )
}
