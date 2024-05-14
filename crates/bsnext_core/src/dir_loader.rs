#![allow(clippy::redundant_pattern_matching)]
#![allow(clippy::single_match)]
use crate::handlers::proxy::ProxyConfig;
use crate::meta::MetaData;
use crate::raw_loader::add_route_layers;
use axum::body::Body;
use axum::extract::{Request, State};

use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use axum::{middleware, Extension, Router};
use bytes::Bytes;
use futures::channel::mpsc::unbounded;
use futures::SinkExt;

use crate::handlers::proxy;
use crate::server::state::ServerState;
use axum::routing::any;
use bsnext_input::route::{DirRoute, ProxyRoute, Route, RouteKind};
use bsnext_resp::{response_modifications_layer, RespMod};
use http::{HeaderValue, StatusCode};
use http_body_util::BodyExt;
use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio_stream::StreamExt;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::decompression::DecompressionLayer;
use tower_http::services::ServeDir;
use tracing::{span, Instrument, Level};

pub async fn serve_dir_loader(
    State(state): State<Arc<ServerState>>,
    req: Request,
    _next: Next,
) -> Response {
    let _path = req.uri().path().to_owned();
    let span = span!(parent: None, Level::INFO, "serve_dir_loader", path = req.uri().path());
    let _guard = span.enter();

    let accepts_html = RespMod::accepts_html(&req);
    let routes = state.routes.read().instrument(span.clone()).await;
    let mut app = Router::new();
    let route_map = routes
        .iter()
        .filter(|r| matches!(&r.kind, RouteKind::Proxy(_) | RouteKind::Dir(_)))
        .fold(HashMap::<&str, Vec<&Route>>::new(), |mut acc, route| {
            acc.entry(route.path.as_str())
                .and_modify(|acc| acc.push(route))
                .or_insert(vec![route]);
            acc
        });

    for (_, routes) in route_map.iter() {
        if routes.len() > 1 {
            tracing::error!(
                "cannot register duplicate dir or proxy routes, only the first will take effect"
            );
        }

        let route = routes.first().expect("guarded");
        let mut router = match &route.kind {
            RouteKind::Proxy(ProxyRoute { proxy }) => {
                let response_handling = if accepts_html {
                    ResponseHandling::Modify
                } else {
                    ResponseHandling::None
                };
                service_for_proxy(response_handling, route).layer(Extension(ProxyConfig {
                    target: proxy.to_owned(),
                    path: route.path.clone(),
                }))
            }
            RouteKind::Dir(DirRoute { dir }) => service_for_dir(route, dir),
            _ => unreachable!("{:?}", route.kind),
        };
        router = add_route_layers(router, route);
        app = app.merge(router);
    }

    drop(routes);

    let r = app.oneshot(req).await;

    r.into_response()
}

fn service_for_dir(route: &Route, dir: &str) -> Router {
    Router::new()
        .nest_service(route.path.as_str(), ServeDir::new(dir))
        .layer(middleware::from_fn(tag_file))
        .layer(middleware::from_fn(response_modifications_layer))
}

async fn tag_file(req: Request, next: Next) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (mut parts, body) = next.run(req).await.into_parts();
    if parts.status.as_u16() == 200 {
        parts.extensions.insert(MetaData::ServedFile);
    }
    Ok(Response::from_parts(parts, body))
}

enum ResponseHandling {
    Modify,
    None,
}

fn service_for_proxy(handling: ResponseHandling, route: &Route) -> Router {
    match handling {
        ResponseHandling::Modify => Router::new()
            .nest_service(route.path.as_str(), any(proxy::proxy_handler))
            .layer(
                ServiceBuilder::new()
                    .layer(middleware::from_fn(response_modifications_layer))
                    .layer(DecompressionLayer::new()),
            ),
        ResponseHandling::None => Router::new()
            .nest_service(route.path.as_str(), any(proxy::proxy_handler))
            .layer(middleware::from_fn(response_modifications_layer)),
    }
}

#[allow(dead_code)]
async fn print_request_response(
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let (parts, body) = req.into_parts();
    let bytes = buffer_and_print("request", body).await?;
    let req = Request::from_parts(parts, Body::from(bytes));

    let res = next.run(req).await;
    let (parts, body) = res.into_parts();

    let (mut sender, rec) = unbounded::<Bytes>();
    let is_event_stream = parts.headers.get("content-type")
        == Some(&HeaderValue::from_str("text/event-stream").unwrap());

    tokio::spawn(async move {
        let mut stream = body.into_data_stream();
        let mut chunks: Vec<Bytes> = vec![];
        while let Some(b) = stream.next().await {
            tracing::info!("CHUNK");
            match b {
                Ok(bytes) => {
                    if is_event_stream {
                        // let _written = file.write(&bytes).await;
                        chunks.push(bytes.to_owned());
                        match sender.send(bytes.to_owned()).await {
                            Ok(_) => tracing::trace!("stripped chunk sent"),
                            Err(_) => tracing::error!("stripped chunk not sent"),
                        };
                    } else {
                        match sender.send(bytes.to_owned()).await {
                            Ok(_) => tracing::trace!("chunk sent"),
                            Err(_) => tracing::error!("not sent"),
                        };
                    }
                }
                Err(e) => {
                    tracing::error!(?e, "error")
                }
            }
        }
        if is_event_stream {
            let path = std::path::Path::new("record").join("out4.yml");
            let mut file = BufWriter::new(File::create(path).await.expect("file"));
            let to_str = chunks
                .into_iter()
                .map(|x| std::str::from_utf8(&x).expect("bytes").to_owned())
                .collect::<Vec<_>>();
            let yml = serde_yaml::to_string(&to_str).expect("to yaml");
            println!("yml={}", yml);
            let _r = file.write(yml.as_bytes());

            match file.flush().await {
                Ok(_) => {}
                Err(_) => {}
            }
        }
    });

    // let res = Response::from_parts(parts, body);
    // while let Some(b) = res.body().next().await {
    //     dbg!(b);
    // }

    // let (parts, body) = res.into_parts();
    // let bytes = buffer_and_print("response", body).await?;
    // let l = lines
    //     .lines()
    //     .map(|l| l.to_owned())
    //     .map(|l| l.strip_prefix("data:").unwrap_or(&l).to_owned())
    //     .filter(|l| !l.trim().is_empty())
    //     .collect::<Vec<_>>();

    let stream = rec
        .throttle(Duration::from_millis(500))
        .map(Ok::<_, Infallible>);

    let res = Response::from_parts(parts, Body::from_stream(stream));

    Ok(res)
}

#[allow(dead_code)]
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
        tracing::debug!("{direction} body = {body:?}");
    }

    Ok(bytes)
}
