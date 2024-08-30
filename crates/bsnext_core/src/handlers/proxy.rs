use anyhow::Context;
use axum::body::Body;
use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use axum::Extension;

use http::{HeaderValue, StatusCode, Uri};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;

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
    let client = {
        req.extensions()
            .get::<Client<HttpsConnector<HttpConnector>, Body>>()
            .expect("must have a client, move this to an extractor?")
    };
    let client_c = client.clone();

    let target = config.target.clone();

    tracing::trace!(?config);

    let path = req.uri().path();

    tracing::trace!(req.uri = %path, config.path = config.path);
    // tracing::trace!(req.headers = ?req.headers());

    let path_query = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or(path);

    let uri = format!("{}{}", target, path_query);
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

    // todo: Which other headers to mod here?
    req.headers_mut().insert("host", host_header_value);

    let res = client_c
        .request(req)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)
        .into_response();

    Ok(res)
}
