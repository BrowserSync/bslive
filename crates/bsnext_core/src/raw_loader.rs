use std::collections::HashMap;
use std::convert::Infallible;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Response, Sse};
use axum::routing::{any, get_service};
use axum::{middleware, Json, Router};
use http::header::CONTENT_TYPE;

use crate::meta::MetaData;
use crate::server::state::ServerState;
use axum::body::Body;
use axum::handler::Handler;
use axum::response::sse::Event;
use bsnext_input::route::{DirRoute, ProxyRoute, Route, RouteKind};
use bytes::Bytes;
use http::{StatusCode, Uri};
use http_body_util::BodyExt;
use std::sync::Arc;
use std::time::Duration;
use tokio_stream::StreamExt;
use tower::ServiceExt;

use crate::common_layers::{add_route_layers, Handling};
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
                    tracing::trace!("temp_router.insert {:?}", path);
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
        // let e = matched.unwrap_err();
        // tracing::trace!(?e, ?attempt, "passing...");
        return next.run(req).await;
    };

    tracing::trace!(?matched, "did match");

    let route = matched.value;
    let _params = matched.params;

    let mut app = match &route.kind {
        RouteKind::Sse { sse } => {
            let raw = sse.to_owned();
            Router::new().fallback_service(any(|| async move {
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
            }))
        }
        RouteKind::Raw { raw } => {
            tracing::trace!("-> served Route::Raw {} {} bytes", route.path, raw.len());
            let moved = raw.clone();
            let p = req.uri().path().to_owned();
            Router::new().fallback_service(any(|| async move { text_asset_response(&p, &moved) }))
        }
        RouteKind::Html { html } => {
            tracing::trace!("-> served Route::Html {} {} bytes", route.path, html.len());
            let moved = html.clone();
            Router::new().fallback_service(any(|| async { Html(moved) }))
        }
        RouteKind::Json { json } => {
            tracing::trace!("-> served Route::Json {} {}", route.path, json);
            let moved = json.to_owned();
            Router::new().fallback_service(any(|| async move { Json(moved) }))
        }
        RouteKind::Dir(DirRoute { dir: _ }) => {
            unreachable!("should never reach RouteKind::Dir")
        }
        RouteKind::Proxy(ProxyRoute { proxy: _ }) => {
            unreachable!("should never reach RouteKind::Proxy")
        }
    };

    app = add_route_layers(app, Handling::Raw, route, &req);
    app.layer(middleware::from_fn(tag_raw))
        .oneshot(req)
        .await
        .into_response()
}

async fn create_raw_router(routes: &[Route]) -> Router {
    let route_map = routes
        .iter()
        .filter(|r| match &r.kind {
            RouteKind::Html { .. } => true,
            RouteKind::Json { .. } => true,
            RouteKind::Raw { .. } => true,
            RouteKind::Sse { .. } => true,
            RouteKind::Proxy(_) => false,
            RouteKind::Dir(_) => false,
        })
        .fold(HashMap::<String, Vec<Route>>::new(), |mut acc, route| {
            acc.entry(route.path.clone())
                .and_modify(|acc| acc.push(route.clone()))
                .or_insert(vec![route.clone()]);
            acc
        });

    async fn serve_raw(uri: Uri, state: State<Vec<Route>>, req: Request) -> Response {
        if state.len() > 1 {
            tracing::error!(
                "more than 1 matching route for {}, only the last will take effect",
                uri
            )
        }
        match state.last() {
            None => StatusCode::NOT_FOUND.into_response(),
            Some(route) => resp_for(uri, route).await.into_response(),
        }
    }

    let mut router = Router::new();
    for (path, route_list) in route_map {
        // todo(alpha):
        router = router.nest_service(&path, get_service(serve_raw.with_state(route_list)));
    }
    router
}

async fn resp_for(uri: Uri, route: &Route) -> impl IntoResponse {
    match &route.kind {
        RouteKind::Html { html } => Html(html.clone()).into_response(),
        RouteKind::Json { .. } => todo!("not supported yet"),
        RouteKind::Raw { raw } => text_asset_response(uri.path(), &raw).into_response(),
        RouteKind::Sse { .. } => todo!("not cupported yet"),
        RouteKind::Proxy(_) => todo!("not cupported yet"),
        RouteKind::Dir(_) => todo!("not cupported yet"),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[tokio::test]
    async fn test_router_from_routes() -> anyhow::Result<()> {
        // Mock a Route. Change this part depending on your specific Route implementation
        let routes: Vec<Route> = vec![
            Route {
                path: "/route1".to_string(),
                kind: RouteKind::Raw {
                    raw: "<h1>Welcome to Route 1</h1>".to_string(),
                },
                ..Default::default()
            },
            Route {
                path: "/route1".to_string(),
                kind: RouteKind::Raw {
                    raw: "<h1>Welcome to Route 1.1</h1>".to_string(),
                },
                ..Default::default()
            },
            Route {
                path: "/route1/abc".to_string(),
                kind: RouteKind::Html {
                    html: "This is HTML".to_string(),
                },
                ..Default::default()
            },
            Route {
                path: "/route4".to_string(),
                kind: RouteKind::Sse {
                    sse: "This is a server-sent event for Route 4".to_string(),
                },
                ..Default::default()
            },
        ];

        let router = create_raw_router(&routes).await;

        // Define the request
        let request = http::Request::builder()
            .uri("/route1")
            .body(Body::empty())
            .unwrap();

        // Make a one-shot request on the router
        let response = router.oneshot(request).await?;
        let (parts, body) = response.into_parts();
        dbg!(parts);
        let response_body = body.collect().await?.to_bytes();
        let response_body_str = std::str::from_utf8(&response_body)?;
        assert_eq!(response_body_str, "<h1>Welcome to Route 1.1</h1>");
        Ok(())
    }
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
