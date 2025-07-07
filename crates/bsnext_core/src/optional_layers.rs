use crate::dynamic_query_params;
use axum::extract::{Request, State};
use axum::middleware::{map_response_with_state, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::MethodRouter;
use axum::{middleware, Extension, Router};
use axum_extra::middleware::option_layer;
use bsnext_input::route::{CompType, CompressionOpts, CorsOpts, DelayKind, DelayOpts, Opts};
use bsnext_resp::{response_modifications_layer, InjectHandling};
use dynamic_query_params::dynamic_query_params_handler;
use http::{HeaderName, HeaderValue};
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::time::Duration;
use tokio::time::sleep;
use tower::ServiceBuilder;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

pub fn optional_layers(app: Router, opts: &Opts) -> Router {
    let mut app = app;
    let cors_enabled_layer = opts
        .cors
        .as_ref()
        .filter(|v| **v == CorsOpts::Cors(true))
        .map(|_| CorsLayer::permissive());

    let compression_layer = opts.compression.as_ref().and_then(comp_opts_to_layer);

    let delay_enabled_layer = opts
        .delay
        .as_ref()
        .map(|delay| middleware::from_fn_with_state(delay.clone(), delay_mw));

    let injections = opts.inject.as_injections();

    let set_response_headers_layer = opts
        .headers
        .as_ref()
        .map(|headers| map_response_with_state(headers.clone(), set_resp_headers_from_strs));

    let headers = opts.cache.as_headers();
    let prevent_cache_headers_layer = map_response_with_state(headers, set_resp_headers);

    let optional_stack = ServiceBuilder::new()
        .layer(middleware::from_fn(dynamic_query_params_handler))
        .layer(middleware::from_fn(response_modifications_layer))
        .layer(prevent_cache_headers_layer)
        .layer(option_layer(set_response_headers_layer))
        .layer(option_layer(cors_enabled_layer))
        .layer(option_layer(delay_enabled_layer));

    app = app.layer(optional_stack);

    // The compression layer has a different type, so needs to apply outside the optional stack
    // this essentially wrapping everything.
    // I'm sure there's a cleaner way...
    if let Some(cl) = compression_layer {
        app = app.layer(cl);
    }

    app.layer(Extension(InjectHandling {
        items: injections.items,
    }))
}
pub fn optional_layers_lol(app: MethodRouter, opts: &Opts) -> MethodRouter {
    let mut app = app;
    let cors_enabled_layer = opts
        .cors
        .as_ref()
        .filter(|v| **v == CorsOpts::Cors(true))
        .map(|_| CorsLayer::permissive());

    let compression_layer = opts.compression.as_ref().and_then(comp_opts_to_layer);

    let delay_enabled_layer = opts
        .delay
        .as_ref()
        .map(|delay| middleware::from_fn_with_state(delay.clone(), delay_mw));

    let injections = opts.inject.as_injections();

    let set_response_headers_layer = opts
        .headers
        .as_ref()
        .map(|headers| map_response_with_state(headers.clone(), set_resp_headers_from_strs));

    let headers = opts.cache.as_headers();
    let prevent_cache_headers_layer = map_response_with_state(headers, set_resp_headers);

    let optional_stack = ServiceBuilder::new()
        .layer(middleware::from_fn(dynamic_query_params_handler))
        .layer(middleware::from_fn(response_modifications_layer))
        .layer(prevent_cache_headers_layer)
        .layer(option_layer(set_response_headers_layer))
        .layer(option_layer(cors_enabled_layer))
        .layer(option_layer(delay_enabled_layer));

    app = app.layer(optional_stack);

    // The compression layer has a different type, so needs to apply outside the optional stack
    // this essentially wrapping everything.
    // I'm sure there's a cleaner way...
    if let Some(cl) = compression_layer {
        app = app.layer(cl);
    }

    app.layer(Extension(InjectHandling {
        items: injections.items,
    }))
}

async fn delay_mw(
    State(delay_opts): State<DelayOpts>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    match delay_opts {
        DelayOpts::Delay(DelayKind::Ms(ms)) => {
            let res = next.run(req).await;
            sleep(Duration::from_millis(ms)).await;
            Ok::<_, Infallible>(res)
        }
    }
}

async fn set_resp_headers_from_strs<B>(
    State(header_map): State<BTreeMap<String, String>>,
    mut response: Response<B>,
) -> Response<B> {
    let headers = response.headers_mut();
    for (k, v) in header_map {
        let hn = HeaderName::from_bytes(k.as_bytes());
        let hv = HeaderValue::from_bytes(v.as_bytes());
        match (hn, hv) {
            (Ok(k), Ok(v)) => {
                tracing::debug!("did insert header `{}`: `{:?}`", k, v);
                headers.insert(k, v);
            }
            (Ok(n), Err(_e)) => {
                tracing::error!("invalid header value: `{}` for name: `{}`", v, n)
            }
            (Err(_e), Ok(..)) => {
                tracing::error!("invalid header name `{}`", k)
            }
            (Err(_e), Err(_e2)) => {
                tracing::error!("invalid header name AND value `{}:{}`", k, v)
            }
        }
    }

    response
}

async fn set_resp_headers<B>(
    State(header_map): State<Vec<(HeaderName, HeaderValue)>>,
    mut response: Response<B>,
) -> Response<B> {
    let headers = response.headers_mut();
    for (k, v) in header_map {
        headers.insert(k, v);
    }
    response
}

fn comp_opts_to_layer(comp: &CompressionOpts) -> Option<CompressionLayer> {
    match comp {
        CompressionOpts::Bool(false) => None,
        CompressionOpts::Bool(true) => Some(CompressionLayer::new()),
        CompressionOpts::CompType(comp_type) => match comp_type {
            CompType::Gzip => Some(
                CompressionLayer::new()
                    .gzip(true)
                    .no_br()
                    .no_deflate()
                    .no_zstd(),
            ),
            CompType::Br => Some(
                CompressionLayer::new()
                    .br(true)
                    .no_gzip()
                    .no_deflate()
                    .no_zstd(),
            ),
            CompType::Deflate => Some(
                CompressionLayer::new()
                    .deflate(true)
                    .no_gzip()
                    .no_br()
                    .no_zstd(),
            ),
            CompType::Zstd => Some(
                CompressionLayer::new()
                    .zstd(true)
                    .no_gzip()
                    .no_deflate()
                    .no_br(),
            ),
        },
    }
}
