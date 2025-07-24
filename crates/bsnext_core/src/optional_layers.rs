use axum::extract::{Request, State};
use axum::middleware::{map_response_with_state, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::MethodRouter;
use axum_extra::middleware::option_layer;
use bsnext_input::route::{CorsOpts, DelayKind, DelayOpts, Opts};
use http::{HeaderName, HeaderValue};
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::time::Duration;
use tokio::time::sleep;
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

pub fn optional_layers(app: MethodRouter, opts: &Opts) -> MethodRouter {
    let mut app = app;
    let cors_enabled_layer = opts
        .cors
        .as_ref()
        .filter(|v| **v == CorsOpts::Cors(true))
        .map(|_| CorsLayer::permissive());

    let set_response_headers_layer = opts
        .headers
        .as_ref()
        .map(|headers| map_response_with_state(headers.clone(), set_resp_headers_from_strs));

    let optional_stack = ServiceBuilder::new()
        .layer(option_layer(set_response_headers_layer))
        .layer(option_layer(cors_enabled_layer));

    app = app.layer(optional_stack);

    // // The compression layer has a different type, so needs to apply outside the optional stack
    // // this essentially wrapping everything.
    // // I'm sure there's a cleaner way...
    // if let Some(cl) = compression_layer {
    //     app = app.layer(cl);
    // }

    app
}

pub async fn delay_mw(
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

async fn set_resp_headers_from_strs(
    State(header_map): State<BTreeMap<String, String>>,
    mut response: Response,
) -> Response {
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

pub async fn set_resp_headers<B>(
    State(header_map): State<Vec<(HeaderName, HeaderValue)>>,
    mut response: Response<B>,
) -> Response<B> {
    let headers = response.headers_mut();
    for (k, v) in header_map {
        headers.insert(k, v);
    }
    response
}
