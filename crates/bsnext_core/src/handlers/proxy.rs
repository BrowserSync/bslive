use anyhow::Context;
use axum::body::Body;
use axum::extract::Request;
use axum::handler::Handler;
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use axum::Extension;
use bsnext_guards::route_guard::RouteGuard;
use bsnext_resp::InjectHandling;
use http::{HeaderValue, StatusCode, Uri};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use tower::ServiceExt;
use tower_http::decompression::DecompressionLayer;
use tracing::{trace_span, Instrument};

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub target: String,
    pub path: String,
}

// Make our own error that wraps `anyhow::Error`.
pub struct AnyAppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AnyAppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}
impl<E> From<E> for AnyAppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub async fn proxy_handler(
    Extension(config): Extension<ProxyConfig>,
    req: Request,
) -> Result<Response, AnyAppError> {
    let target = config.target.clone();
    let path = req.uri().path();

    let span = trace_span!("proxy_handler");
    let _g = span.enter();

    tracing::trace!(?config);

    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    let uri = format!("{target}{path_query}");
    let parsed = Uri::try_from(uri).context("tried to parse")?;

    let target = match (parsed.host(), parsed.port()) {
        (Some(host), Some(port)) => format!("{host}:{port}"),
        (Some(host), None) => host.to_owned(),
        _ => unreachable!("could not extract `host` from url"),
    };

    tracing::trace!(outgoing.uri = %parsed);

    let host_header_value = HeaderValue::from_str(&target).unwrap();

    let (parts, body) = req.into_parts();
    let mut req = Request::from_parts(parts, body);

    *req.uri_mut() = parsed;

    // todo(alpha): Which other headers to mod here?
    req.headers_mut().insert("host", host_header_value);
    req.headers_mut().remove("referer");

    // decompress requests if needed
    if let Some(h) = req.extensions().get::<InjectHandling>() {
        let req_accepted = h.items.iter().any(|item| item.accept_req(&req));
        tracing::trace!(
            req.accepted = req_accepted,
            req.accept.header = req
                .headers()
                .get("accept")
                .map(|h| h.to_str().unwrap_or("")),
            "will accept request + decompress?"
        );
        if req_accepted {
            let sv2 = any(serve_one_proxy_req.layer(DecompressionLayer::new()));
            return Ok(sv2
                .oneshot(req)
                .instrument(span.clone())
                .await
                .into_response());
        }
    }

    let sv2 = any(serve_one_proxy_req);
    Ok(sv2
        .oneshot(req)
        .instrument(span.clone())
        .await
        .into_response())
}

async fn serve_one_proxy_req(req: Request) -> Response {
    tracing::trace!("serve_one_proxy_req {}", req.uri().to_string());
    let client = {
        req.extensions()
            .get::<Client<HttpsConnector<HttpConnector>, Body>>()
            .expect("must have a client, move this to an extractor?")
    };
    tracing::trace!(req.headers = ?req.headers());
    let client_c = client.clone();
    let resp = client_c
        .request(req)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)
        .into_response();
    tracing::trace!(resp.headers = ?resp.headers());
    resp
}
