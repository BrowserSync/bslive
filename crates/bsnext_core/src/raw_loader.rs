use std::collections::HashMap;
use std::convert::Infallible;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Response, Sse};
use axum::routing::{any, any_service, get_service};
use axum::{middleware, Json, Router};
use http::header::CONTENT_TYPE;

use crate::meta::MetaData;
use crate::server::state::ServerState;
use axum::body::Body;
use axum::handler::{Handler, HandlerWithoutStateExt};
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

    let raw_router = app.raw_router.read().await;

    raw_router
        .clone()
        .layer(middleware::from_fn(tag_raw))
        .oneshot(req)
        .await
        .into_response()
}

pub fn create_raw_router(routes: &[Route]) -> Router {
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
        tracing::trace!("serve_raw {}", req.uri().to_string());
        if state.len() > 1 {
            tracing::error!(
                "more than 1 matching route for {}, only the last will take effect",
                uri
            )
        }
        match state.last() {
            None => StatusCode::NOT_FOUND.into_response(),
            Some(route) => raw_resp_for(uri, route).await.into_response(),
        }
    }

    let mut router = Router::new();
    for (path, route_list) in route_map {
        // todo(alpha): support more use-cases here?
        router = router.route_service(&path, any_service(serve_raw.with_state(route_list)));
    }
    router
}

async fn raw_resp_for(uri: Uri, route: &Route) -> impl IntoResponse {
    match &route.kind {
        RouteKind::Html { html } => Html(html.clone()).into_response(),
        RouteKind::Json { json } => Json(&json.0).into_response(),
        RouteKind::Raw { raw } => text_asset_response(uri.path(), &raw).into_response(),
        RouteKind::Sse { sse } => {
            let l = sse
                .lines()
                .map(|l| l.to_owned())
                .map(|l| l.strip_prefix("data:").unwrap_or(&l).to_owned())
                .filter(|l| !l.trim().is_empty())
                .collect::<Vec<_>>();

            tracing::trace!(lines.count = l.len(), "sending EventStream");

            let stream = tokio_stream::iter(l)
                .throttle(Duration::from_millis(10))
                .map(|chu| Event::default().data(chu))
                .map(Ok::<_, Infallible>);

            Sse::new(stream).into_response()
        }
        RouteKind::Proxy(_) => todo!("not cupported yet"),
        RouteKind::Dir(_) => todo!("not cupported yet"),
    }
}

#[cfg(test)]
mod raw_test {
    use super::*;
    use crate::server::router::common::to_resp_parts_and_body;

    #[tokio::test]
    async fn duplicate_path() -> anyhow::Result<()> {
        let routes_input = r#"
            - path: /route1
              html: <h1>Welcome to Route 1</h1>
            - path: /route1
              html: <h1>Welcome to Route 1.1</h1>
            - path: /raw1
              raw: raw1
            - path: /json
              json: [1]
            - path: /sse
              sse: |
                a
                b
                c"#;

        {
            let routes: Vec<Route> = serde_yaml::from_str(routes_input)?;
            let router = create_raw_router(&routes);
            // Define the request
            let request = Request::get("/route1").body(Body::empty())?;
            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;
            let (parts, body) = to_resp_parts_and_body(response).await;
            assert_eq!(body, "<h1>Welcome to Route 1.1</h1>");
        }

        {
            let routes: Vec<Route> = serde_yaml::from_str(routes_input)?;
            let router = create_raw_router(&routes);
            // Define the request
            let request = Request::get("/raw1").body(Body::empty())?;
            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;
            let (parts, body) = to_resp_parts_and_body(response).await;
            assert_eq!(body, "raw1");
        }

        {
            let routes: Vec<Route> = serde_yaml::from_str(routes_input)?;
            let router = create_raw_router(&routes);
            // Define the request
            let request = Request::get("/json").body(Body::empty())?;
            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;
            let (parts, body) = to_resp_parts_and_body(response).await;
            assert_eq!(body, "[1]");
        }

        {
            let routes: Vec<Route> = serde_yaml::from_str(routes_input)?;
            let router = create_raw_router(&routes);
            // Define the request
            let request = Request::get("/sse").body(Body::empty())?;
            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;
            let (parts, body) = to_resp_parts_and_body(response).await;
            let lines = body
                .lines()
                .filter(|x| !x.trim().is_empty())
                .collect::<Vec<_>>();
            assert_eq!("data: a,data: b,data: c", lines.join(","));
        }

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

#[tracing::instrument]
pub fn text_asset_response(path: &str, content: &str) -> Response {
    let mime = mime_guess::from_path(path);
    tracing::trace!(?mime, ?path);
    let aas_str = mime.first_or_text_plain();
    let cloned = content.to_owned();
    ([(CONTENT_TYPE, aas_str.to_string())], cloned).into_response()
}
