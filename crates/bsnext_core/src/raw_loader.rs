use std::convert::Infallible;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Response, Sse};
use axum::routing::any;
use axum::{middleware, Json, Router};
use http::header::CONTENT_TYPE;

use crate::meta::MetaData;
use crate::server::state::ServerState;
use axum::body::Body;
use axum::response::sse::Event;
use bsnext_input::route::{DirRoute, ProxyRoute, RouteKind};
use bsnext_resp::response_modifications_layer;
use bytes::Bytes;
use http::StatusCode;
use http_body_util::BodyExt;
use std::sync::Arc;
use std::time::Duration;

use tokio_stream::StreamExt;
use tower::ServiceExt;

use crate::common_layers::add_route_layers;
use tracing::{span, Level};

// use futures_util::stream::{self, Stream};

pub async fn raw_loader(
    State(app): State<Arc<ServerState>>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    let span = span!(parent: None, Level::INFO, "raw_loader", path = req.uri().path());
    let _guard = span.enter();

    let routes = app.routes.read().await;
    let mut temp_router = matchit::Router::new();
    // let mut app = Router::new();

    // which route kinds can be hard-matched first
    for route in routes.iter() {
        let path = route.path();
        match &route.kind {
            RouteKind::Html { .. }
            | RouteKind::Json { .. }
            | RouteKind::Raw { .. }
            | RouteKind::Sse { .. } => {
                let existing = temp_router.at_mut(path);
                if let Ok(prev) = existing {
                    *prev.value = route.clone();
                    tracing::trace!("updated mutable route at {}", path)
                } else if let Err(err) = existing {
                    match temp_router.insert(path, route.clone()) {
                        Ok(_) => {
                            tracing::trace!(path, "+")
                        }
                        Err(_) => {
                            tracing::error!("❌ {:?}", err.to_string())
                        }
                    }
                }
            }
            _ => {}
        }
    }

    drop(routes);

    let matched = temp_router.at(req.uri().path());

    let Ok(matched) = matched else {
        return next.run(req).await;
    };

    let route = matched.value;
    let _params = matched.params;

    let mut app = Router::new();
    match &route.kind {
        RouteKind::Sse { sse } => {
            let raw = sse.to_owned();
            app = app.route(
                req.uri().path(),
                any(|| async move {
                    let l = raw
                        .lines()
                        .map(|l| l.to_owned())
                        .map(|l| l.strip_prefix("data:").unwrap_or(&l).to_owned())
                        .filter(|l| !l.trim().is_empty())
                        .collect::<Vec<_>>();

                    tracing::trace!(lines.count = l.len(), "sending EventStream");

                    let stream = tokio_stream::iter(l)
                        .throttle(Duration::from_millis(500))
                        .map(|chu| Event::default().data(chu))
                        .map(Ok::<_, Infallible>);

                    Sse::new(stream)
                }),
            )
        }
        RouteKind::Raw { raw } => {
            tracing::trace!("-> served Route::Raw {} {} bytes", route.path, raw.len());
            let moved = raw.clone();
            let p = req.uri().path().to_owned();
            app = app.route(
                req.uri().path(),
                any(|| async move { text_asset_response(&p, &moved) }),
            );
        }
        RouteKind::Html { html } => {
            tracing::trace!("-> served Route::Html {} {} bytes", route.path, html.len());
            let moved = html.clone();
            app = app.route(req.uri().path(), any(|| async { Html(moved) }));
        }
        RouteKind::Json { json } => {
            tracing::trace!("-> served Route::Json {} {}", route.path, json);
            let moved = json.to_owned();
            app = app.route(req.uri().path(), any(|| async move { Json(moved) }));
        }
        RouteKind::Dir(DirRoute { dir: _ }) => {
            unreachable!("should never reach RouteKind::Dir")
        }
        RouteKind::Proxy(ProxyRoute { proxy: _ }) => {
            unreachable!("should never reach RouteKind::Proxy")
        }
    }

    app = add_route_layers(app, route);

    app.layer(middleware::from_fn(tag_raw))
        .layer(middleware::from_fn(response_modifications_layer))
        .oneshot(req)
        .await
        .into_response()
}

async fn tag_raw(req: Request, next: Next) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (mut parts, body) = next.run(req).await.into_parts();
    if parts.status.as_u16() == 200 {
        parts.extensions.insert(MetaData::ServedRaw);
    }
    Ok(Response::from_parts(parts, body))
}

#[allow(dead_code)]
async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let res = next.run(req).await;

    let (parts, body) = res.into_parts();
    let bytes = buffer_and_print("response", body).await?;
    let lines = bytes.chunks(150).map(|c| c.to_owned()).collect::<Vec<_>>();

    let stream = tokio_stream::iter(lines)
        .throttle(Duration::from_millis(500))
        .map(Ok::<_, Infallible>);

    let res = Response::from_parts(parts, Body::from_stream(stream));

    Ok(res)
}

async fn buffer_and_print<B>(direction: &str, body: B) -> Result<Bytes, (StatusCode, String)>
where
    B: axum::body::HttpBody<Data = Bytes>,
    B::Error: std::fmt::Display,
{
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read {direction} body: {err}"),
            ));
        }
    };

    if let Ok(body) = std::str::from_utf8(&bytes) {
        tracing::debug!("▶️ {direction} body = {body:?}");
    }

    Ok(bytes)
}

pub fn text_asset_response(path: &str, css: &str) -> Response {
    let mime = mime_guess::from_path(path);
    let aas_str = mime.first_or_text_plain();
    let cloned = css.to_owned();
    ([(CONTENT_TYPE, aas_str.to_string())], cloned).into_response()
}
