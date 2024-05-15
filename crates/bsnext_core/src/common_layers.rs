use axum::extract::Request;
use axum::middleware::Next;
use axum::{middleware, Router};
use http::{HeaderName, HeaderValue};
use std::convert::Infallible;
use std::time::Duration;

use bsnext_input::route::{CorsOpts, DelayKind, DelayOpts, Route};
use tokio::time::sleep;
use tower_http::cors::CorsLayer;
use tower_http::set_header;

pub fn add_route_layers(app: Router, route: &Route) -> Router {
    let mut app = app;

    if route
        .cors_opts
        .as_ref()
        .is_some_and(|v| *v == CorsOpts::Cors(true))
    {
        tracing::trace!(to = route.path, "adding permissive cors");
        app = app.layer(CorsLayer::permissive());
    }

    if let Some(DelayOpts::Delay(DelayKind::Ms(ms))) = route.delay_opts.as_ref() {
        tracing::trace!(to = route.path, ?ms, "adding a delay");
        let ms = *ms;
        app = app.layer(middleware::from_fn(
            move |req: Request, next: Next| async move {
                let res = next.run(req).await;
                sleep(Duration::from_millis(ms)).await;
                Ok::<_, Infallible>(res)
            },
        ));
    }

    if let Some(headers) = route.headers.as_ref() {
        for (k, v) in headers {
            let hn = HeaderName::from_bytes(k.as_bytes());
            let hv = HeaderValue::from_bytes(v.as_bytes());
            match (hn, hv) {
                (Ok(n), Ok(v)) => {
                    app = app.layer(set_header::SetResponseHeaderLayer::overriding(n, v));
                }
                (Ok(_), Err(_e)) => {
                    tracing::error!("invalid header value `{}`", v)
                }
                (Err(_e), Ok(_)) => {
                    tracing::error!("invalid header name `{}`", k)
                }
                (Err(_e), Err(_e2)) => {
                    tracing::error!("invalid header name AND value `{}:{}`", k, v)
                }
            }
        }
    }
    // if route.opts.as_ref().is_some_and(|v| v.buff) {
    // app = app.layer(middleware::from_fn(print_request_response));
    // }

    app
}
