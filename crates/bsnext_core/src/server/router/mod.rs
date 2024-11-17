pub mod common;

use crate::panic_handler::handle_panic;
use axum::extract::{Request, State};
use axum::http::Uri;
use axum::middleware::{from_fn_with_state, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, MethodRouter};
use axum::{http, middleware, Extension, Router};

use axum::body::Body;
use http::header::CONTENT_TYPE;
use http::{HeaderValue, StatusCode};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use mime_guess::mime;
use std::sync::Arc;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::catch_panic::CatchPanicLayer;

use crate::meta::MetaData;
use crate::server::router::assets::pub_ui_assets;
use crate::server::router::pub_api::pub_api;
use crate::server::state::ServerState;
use crate::ws::ws_handler;
use bsnext_client::html_with_base;
use bsnext_dto::{RouteDTO, ServerDesc};
use tracing::{span, Level};

mod assets;
mod pub_api;

pub fn make_router(state: &Arc<ServerState>) -> Router {
    let https = HttpsConnector::new();
    let client: Client<HttpsConnector<HttpConnector>, Body> =
        Client::builder(TokioExecutor::new()).build(https);

    let router = Router::new()
        .merge(built_ins(state.clone()))
        .merge(dynamic_loaders(state.clone()));

    router
        // todo(alpha): enable tracing on a per-item basis?
        // .layer(TraceLayer::new_for_http())
        // todo(alpha): re-enable in the correct place?
        // .layer(TimeoutLayer::new(Duration::from_secs(10)))
        .layer(Extension(client))
    // todo: When to add this compression back in?
    // .layer(CompressionLayer::new())
}

pub fn built_ins(state: Arc<ServerState>) -> Router {
    async fn handler(State(app): State<Arc<ServerState>>, _uri: Uri) -> impl IntoResponse {
        // let v = app.lock().await;
        let routes = app.routes.read().await;
        let _dto = ServerDesc {
            routes: routes.iter().map(RouteDTO::from).collect(),
            id: app.id.to_string(),
        };
        let markup = html_with_base("/__bs_assets/ui/");
        (
            [(
                CONTENT_TYPE,
                HeaderValue::from_static(mime::TEXT_HTML_UTF_8.as_ref()),
            )],
            markup,
        )
            .into_response()
    }
    async fn js_handler(_uri: Uri) -> impl IntoResponse {
        let markup = include_str!("../../../../../inject/dist/index.js");
        (
            [(
                CONTENT_TYPE,
                HeaderValue::from_static(mime::APPLICATION_JAVASCRIPT_UTF_8.as_ref()),
            )],
            markup,
        )
            .into_response()
    }

    route("/__bslive", get(handler))
        .route("/__bs_js", get(js_handler))
        .route("/__bs_ws", get(ws_handler))
        .nest("/__bs_api", pub_api(state.clone()))
        .nest("/__bs_assets/ui", pub_ui_assets(state.clone()))
        .with_state(state.clone())
}

fn route(path: &str, method_router: MethodRouter<Arc<ServerState>>) -> Router<Arc<ServerState>> {
    Router::new().route(path, method_router)
}

pub fn dynamic_loaders(state: Arc<ServerState>) -> Router {
    Router::new()
        .layer(
            ServiceBuilder::new()
                .layer(from_fn_with_state(state.clone(), tagging_layer))
                // .layer(from_fn_with_state(state.clone(), maybe_proxy))
                // todo(alpha): have the order listed here instead: static -> dir -> proxy
                // .layer(from_fn_with_state(state.clone(), not_found_loader))
                .layer(from_fn_with_state(state.clone(), dynamic_router)),
        )
        .layer(CatchPanicLayer::custom(handle_panic))
        .with_state(state.clone())
}

async fn tagging_layer(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let mut r = next.run(req).await;
    #[allow(unused_variables)]
    if let Some(md) = r.extensions_mut().get::<MetaData>() {
        // r.headers_mut()
        //     .insert("abc", HeaderValue::from_static(md.as_header_value()));
    }
    Ok(r)
}

#[derive(serde::Deserialize, Debug)]
pub enum Encoding {
    Br,
    Zip,
}

pub async fn dynamic_router(
    State(app): State<Arc<ServerState>>,
    req: Request,
    _next: Next,
) -> impl IntoResponse {
    let span = span!(parent: None, Level::INFO, "raw_loader", path = req.uri().path());
    let _guard = span.enter();

    let raw_router = app.raw_router.read().await;

    raw_router
        .clone()
        .layer(middleware::from_fn(tag_dynamic))
        .oneshot(req)
        .await
        .into_response()
}

async fn tag_dynamic(req: Request, next: Next) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (mut parts, body) = next.run(req).await.into_parts();
    if parts.status.as_u16() == 200 {
        parts.extensions.insert(MetaData::ServedRaw);
    }
    Ok(Response::from_parts(parts, body))
}
