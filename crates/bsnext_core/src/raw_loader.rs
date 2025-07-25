use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Response, Sse};
use axum::{Extension, Json};
use http::header::CONTENT_TYPE;
use std::convert::Infallible;
use std::fs;

use axum::body::Body;
use axum::response::sse::Event;
use bsnext_guards::OuterUri;
use bsnext_input::route::{RawRoute, SseOpts};
use bytes::Bytes;
use http::{StatusCode, Uri};
use http_body_util::BodyExt;
use std::time::Duration;
use tokio_stream::StreamExt;

pub async fn serve_raw_one(
    uri: Uri,
    Extension(outer_uri): Extension<OuterUri>,
    state: State<RawRoute>,
    _req: Request,
) -> Response {
    tracing::trace!(?outer_uri, ?uri, "serve_raw_one");
    raw_resp_for(outer_uri, &state.0).await.into_response()
}

pub async fn raw_resp_for(outer_uri: OuterUri, route: &RawRoute) -> impl IntoResponse {
    let uri = outer_uri.0;
    match route {
        RawRoute::Html { html } => {
            tracing::trace!("raw_resp_for will respond with HTML");
            Html(html.clone()).into_response()
        }
        RawRoute::Json { json } => Json(&json.0).into_response(),
        RawRoute::Raw { raw } => text_asset_response(uri.path(), raw).into_response(),
        RawRoute::Sse {
            sse: SseOpts { body, throttle_ms },
        } => {
            let file_prefix = body.split_once("file:");
            let body = match file_prefix {
                None => body.to_owned(),
                Some((_, path)) => {
                    let span = tracing::debug_span!("reading SSE body content", path = path);
                    let _g = span.enter();
                    tracing::debug!(?path, "will read sse file content");
                    match fs::read_to_string(path) {
                        Ok(str) => {
                            tracing::debug!("did read {} bytes", str.len());
                            str
                        }
                        Err(err) => {
                            tracing::error!(?err, ?path);
                            return (
                                StatusCode::NOT_FOUND,
                                format!("{path} was not found. err {err}"),
                            )
                                .into_response();
                        }
                    }
                }
            };
            let iter = body
                .lines()
                .map(|l| {
                    l.trim()
                        .strip_prefix("data:")
                        .unwrap_or(l)
                        .trim()
                        .to_owned()
                })
                .filter(|l| !l.is_empty())
                .collect::<Vec<_>>();

            tracing::trace!(lines.count = iter.len(), "sending EventStream");

            let milli = throttle_ms.unwrap_or(10);
            let stream = tokio_stream::iter(iter)
                .throttle(Duration::from_millis(milli))
                .map(|chu| Event::default().data(chu))
                .map(Ok::<_, Infallible>);

            Sse::new(stream).into_response()
        }
    }
}

#[cfg(test)]
mod raw_test {
    use super::*;
    use crate::handler_stack::RouteMap;
    use crate::runtime_ctx::RuntimeCtx;
    use crate::server::router::common::to_resp_parts_and_body;
    use bsnext_input::route::Route;
    use tower::ServiceExt;

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
              sse:
                body: |
                  a
                  b
                  c"#;

        {
            let routes: Vec<Route> = serde_yaml::from_str(routes_input)?;
            let router = RouteMap::new_from_routes(&routes).into_router(&RuntimeCtx::default());
            // Define the request
            let request = Request::get("/route1").body(Body::empty())?;
            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;
            let (_parts, body) = to_resp_parts_and_body(response).await;
            assert_eq!(body, "<h1>Welcome to Route 1</h1>");
        }

        {
            let routes: Vec<Route> = serde_yaml::from_str(routes_input)?;
            let router = RouteMap::new_from_routes(&routes).into_router(&RuntimeCtx::default());
            // Define the request
            let request = Request::get("/raw1").body(Body::empty())?;
            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;
            let (_parts, body) = to_resp_parts_and_body(response).await;
            assert_eq!(body, "raw1");
        }

        {
            let routes: Vec<Route> = serde_yaml::from_str(routes_input)?;
            let router = RouteMap::new_from_routes(&routes).into_router(&RuntimeCtx::default());
            // Define the request
            let request = Request::get("/json").body(Body::empty())?;
            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;
            let (_parts, body) = to_resp_parts_and_body(response).await;
            assert_eq!(body, "[1]");
        }

        {
            let routes: Vec<Route> = serde_yaml::from_str(routes_input)?;
            let router = RouteMap::new_from_routes(&routes).into_router(&RuntimeCtx::default());
            // Define the request
            let request = Request::get("/sse").body(Body::empty())?;
            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;
            let (_parts, body) = to_resp_parts_and_body(response).await;
            let lines = body
                .lines()
                .filter(|x| !x.trim().is_empty())
                .collect::<Vec<_>>();
            assert_eq!("data: a,data: b,data: c", lines.join(","));
        }

        Ok(())
    }
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
