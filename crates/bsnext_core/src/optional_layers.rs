use axum::extract::{Request, State};
use axum::handler::Handler;
use axum::middleware::{map_response_with_state, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::MethodRouter;
use axum::{middleware, Extension};
use axum_extra::middleware::option_layer;
use bsnext_input::route::{CorsOpts, DelayKind, DelayOpts, Opts};
use bsnext_resp::{response_modifications_layer, InjectHandling};
use http::{HeaderName, HeaderValue};
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::time::Duration;
use tokio::time::sleep;
use tower::{Layer, ServiceBuilder};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

pub fn optional_layers(app: MethodRouter, opts: &Opts) -> MethodRouter {
    let app = app;
    let cors_enabled_layer = opts
        .cors
        .as_ref()
        .filter(|v| **v == CorsOpts::Cors(true))
        .map(|_| CorsLayer::permissive());

    let delay_enabled_layer = opts
        .delay
        .as_ref()
        .map(|delay| middleware::from_fn_with_state(delay.clone(), delay_mw));

    let injections = opts.inject.as_injections();
    let inject_layer = Some(injections.items.len())
        .filter(|inj| *inj > 0)
        .map(|_| middleware::from_fn(response_modifications_layer));

    let set_response_headers_layer = opts
        .headers
        .as_ref()
        .map(|headers| map_response_with_state(headers.clone(), set_resp_headers));

    let optional_stack = ServiceBuilder::new()
        .layer(CompressionLayer::new())
        .layer(option_layer(inject_layer))
        .layer(option_layer(set_response_headers_layer))
        .layer(option_layer(cors_enabled_layer))
        .layer(option_layer(delay_enabled_layer));

    app.layer::<_, Infallible>(optional_stack)
        .layer(Extension(InjectHandling {
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

async fn set_resp_headers<B>(
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
            (Err(_e), Ok(v)) => {
                tracing::error!("invalid header name `{}`", k)
            }
            (Err(_e), Err(_e2)) => {
                tracing::error!("invalid header name AND value `{}:{}`", k, v)
            }
        }
    }

    response
}
