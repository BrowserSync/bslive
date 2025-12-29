use crate::server::state::ServerState;
use anyhow::Context;
use axum::body::Body;
use axum::extract::ws::rejection::WebSocketUpgradeRejection;
use axum::extract::{ConnectInfo, Request, State};
use axum::handler::Handler;
use axum::response::{IntoResponse, Response};
use axum::routing::any;
use axum::Extension;
use bsnext_guards::OuterUri;
use http::header::{HOST, ORIGIN, SET_COOKIE};
use http::uri::{Parts, PathAndQuery};
use http::{HeaderName, HeaderValue, StatusCode, Uri};
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use std::collections::BTreeMap;
use std::net::SocketAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use tower::ServiceExt;
use tracing::{trace_span, Instrument};

#[derive(Debug, Clone)]
pub struct ProxyConfig {
    pub target: String,
    pub path: String,
    pub headers: BTreeMap<String, String>,
    pub rewrite_kind: RewriteKind,
    pub socket_addr: Option<SocketAddr>,
}

#[derive(Debug, Clone)]
pub enum RewriteKind {
    Alias,
    Nested,
}

#[derive(Clone, Debug)]
struct ProxyHost {
    host: Option<String>,
}

impl From<Option<bool>> for RewriteKind {
    fn from(value: Option<bool>) -> Self {
        match value {
            None | Some(true) => RewriteKind::Nested,
            Some(false) => RewriteKind::Alias,
        }
    }
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
    Extension(OuterUri(outer_uri)): Extension<OuterUri>,
    Extension(socket): Extension<Arc<Mutex<Option<SocketAddr>>>>,
    uri: Uri,
    req: Request,
) -> Result<Response, AnyAppError> {
    let span = trace_span!("proxy_handler");
    let _g = span.enter();

    tracing::trace!(runtime.socket = ?socket);
    tracing::trace!(runtime.outer_uri = ?outer_uri);
    tracing::trace!(runtime.uri = ?uri);
    tracing::trace!(config.path = ?config.path);
    tracing::trace!(config.target = ?config.target);
    tracing::trace!(config.headers = ?config.headers);
    tracing::trace!(config.rewrite_kind = ?config.rewrite_kind);

    let target = config.target.clone();
    let target_uri = Uri::try_from(&target)?;

    tracing::trace!(target = ?target);
    tracing::trace!(target.uri = ?target_uri);
    tracing::trace!(target.uri.authority = ?target_uri.authority());
    tracing::trace!(target.uri.path_and_query = ?target_uri.path_and_query());

    let args = IntoTarget {
        target_uri: &target_uri,
        outer_uri: &outer_uri,
        uri: &uri,
        config: &config,
    };
    let host = req
        .headers()
        .get(HOST)
        .and_then(|x| x.to_str().ok())
        .map(ToOwned::to_owned);

    let parsed = into_target_uri(args)?;
    tracing::trace!(outgoing.uri = %parsed);
    tracing::trace!(incoming.host = ?host);

    let (parts, body) = req.into_parts();

    let mut req = Request::from_parts(parts, body);

    let target = match (parsed.host(), parsed.port()) {
        (Some(host), Some(port)) => format!("{host}:{port}"),
        (Some(host), None) => host.to_owned(),
        _ => unreachable!("could not extract `host` from url"),
    };

    *req.uri_mut() = parsed;
    let host_header_value = HeaderValue::from_str(&target)?;
    req.headers_mut().insert("host", host_header_value);
    req.headers_mut().remove("referer");
    req.extensions_mut().insert(ProxyHost { host });

    for (k, v) in config.headers {
        match (
            HeaderName::from_bytes(k.as_bytes()),
            HeaderValue::from_bytes(v.as_bytes()),
        ) {
            (Ok(hn), Ok(hv)) => {
                tracing::trace!(?hn, ?hv, "add cookie to outgoing request");
                req.headers_mut().insert(hn, hv);
            }
            _ => {
                // noop
            }
        }
    }

    let sv2 = any(serve_one_proxy_req);
    Ok(sv2
        .oneshot(req)
        .instrument(span.clone())
        .await
        .into_response())
}

#[tracing::instrument(skip_all)]
async fn serve_one_proxy_req(
    Extension(ProxyHost { host }): Extension<ProxyHost>,
    req: Request,
) -> Response {
    tracing::trace!("serve_one_proxy_req {}", req.uri().to_string());
    let client = {
        req.extensions()
            .get::<Client<HttpsConnector<HttpConnector>, Body>>()
            .expect("must have a client, move this to an extractor?")
    };
    tracing::trace!(req.headers = ?req.headers());
    tracing::trace!(req.method = ?req.method());

    let client_c = client.clone();

    let mut resp = client_c
        .request(req)
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)
        .into_response();

    tracing::trace!(resp.status = resp.status().as_u16());
    tracing::trace!(resp.headers = ?resp.headers());

    if let (Some(mut hv), Some(host)) = (resp.headers_mut().get_mut(SET_COOKIE), host) {
        let mut next = hv
            .to_str()
            .ok()
            .map(cookie::Cookie::parse)
            .and_then(|c| c.ok())
            .filter(|x| x.domain().is_some());

        if let Some(next) = next.as_mut() {
            next.set_domain(host.as_str());
            *hv = HeaderValue::from_str(next.to_string().as_str()).unwrap()
        }
    }

    resp
}

struct IntoTarget<'a> {
    /// The base destination URI where the request will be forwarded.
    /// Example: `http://example.com/api`
    target_uri: &'a Uri,
    /// The full original URI as received by the proxy server, including any prefixes.
    /// Used when `RewriteKind::Alias` is active to preserve the full path.
    /// Example: `http://localhost:3000/proxy-path/sub-resource`
    outer_uri: &'a Uri,
    /// The URI after internal routing, usually with the proxy prefix stripped.
    /// Used when `RewriteKind::Nested` is active to append only the sub-path.
    /// Example: `/sub-resource`
    uri: &'a Uri,
    config: &'a ProxyConfig,
}

/// Constructs the final outgoing URI for the proxied request.
///
/// It determines the "source" path based on `config.rewrite_kind`:
/// - `RewriteKind::Alias`: Uses `outer_uri` (preserves the original incoming path).
/// - `RewriteKind::Nested`: Uses `uri` (uses the stripped/sub-path).
///
/// The resulting path is a combination of the `target_uri` path and the selected source path,
/// preserving query parameters from the source.
///
/// ### Examples
///
/// | `target_uri`             | `outer_uri`       | `uri`          | `rewrite_kind` | Result                                    |
/// | :------------------------ | :---------------- | :------------- | :------------- | :---------------------------------------- |
/// | `http://example.com`      | `/api/abc`        | `/abc`         | `Alias`        | `http://example.com/api/abc`              |
/// | `http://example.com`      | `/api/abc`        | `/abc`         | `Nested`       | `http://example.com/abc`                  |
/// | `http://example.com/api`  | `.../test`        | `/users/123`   | `Nested`       | `http://example.com/api/users/123`        |
/// | `http://example.com/api`  | `.../test/u/123`  | `/u/123`       | `Alias`        | `http://example.com/api/test/u/123`       |
/// | `http://example.com/api`  | `.../t?f=b`       | `/u/123?f=b`   | `Nested`       | `http://example.com/api/users/123?f=b`    |
fn into_target_uri(
    IntoTarget {
        target_uri,
        outer_uri,
        uri,
        config,
    }: IntoTarget,
) -> anyhow::Result<Uri> {
    let uri_src = match config.rewrite_kind {
        RewriteKind::Alias => &outer_uri,
        RewriteKind::Nested => &uri,
    };

    let req_query = uri_src.path_and_query().and_then(|x| x.query());
    let target = target_uri.path_and_query().filter(|x| x.path() != "/");
    let src = uri_src.path_and_query().filter(|x| x.path() != "/");

    let next_pq = match (target, src) {
        (Some(target), Some(req)) => {
            let p1 = target.path();
            let p2 = req.path();
            let next = match req_query {
                None => format!("{p1}{p2}"),
                Some(q) => format!("{p1}{p2}?{q}"),
            };
            let v =
                PathAndQuery::from_str(&next).unwrap_or(uri_src.path_and_query().unwrap().clone());
            v
        }
        (Some(target_only), None) => {
            let path = target_only.path();

            let next = match req_query {
                None => path,
                Some(q) => &format!("{path}?{q}"),
            };
            let v =
                PathAndQuery::from_str(next).unwrap_or(uri_src.path_and_query().unwrap().clone());
            v
        }
        (None, Some(req_only)) => req_only.to_owned(),
        (None, None) => uri_src.path_and_query().unwrap().clone(),
    };

    let mut next_parts = Parts::default();
    if let Some(scheme) = target_uri.scheme() {
        next_parts.scheme = Some(scheme.to_owned())
    }
    if let Some(auth) = target_uri.authority() {
        next_parts.authority = Some(auth.to_owned());
    }

    next_parts.path_and_query = Some(next_pq);

    Uri::from_parts(next_parts).context("tried to parse")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    struct TestCase<'a> {
        target: &'a str,
        outer: &'a str,
        uri: &'a str,
        rewrite: RewriteKind,
        expect: &'a str,
    }

    fn check(case: TestCase) {
        let target_uri = Uri::from_str(case.target).unwrap();
        let outer_uri = Uri::from_str(case.outer).unwrap();
        let uri = Uri::from_str(case.uri).unwrap();
        let config = ProxyConfig {
            target: case.target.to_string(),
            path: String::new(),
            headers: BTreeMap::new(),
            rewrite_kind: case.rewrite,
            socket_addr: None,
        };

        let args = IntoTarget {
            target_uri: &target_uri,
            outer_uri: &outer_uri,
            uri: &uri,
            config: &config,
        };

        let result = into_target_uri(args).unwrap();
        assert_eq!(result.to_string(), case.expect);
    }

    #[test]
    fn test_into_target_uri_basic() {
        check(TestCase {
            target: "http://example.com",
            outer: "/",
            uri: "/",
            rewrite: RewriteKind::Alias,
            expect: "http://example.com/",
        });
    }

    #[test]
    fn test_into_target_uri_basic_nested_alias() {
        // the `/api` section is preserved here
        check(TestCase {
            target: "http://example.com",
            outer: "/api/abc",
            uri: "/abc",
            rewrite: RewriteKind::Alias,
            expect: "http://example.com/api/abc",
        });
    }

    #[test]
    fn test_into_target_uri_basic_nested_rewrite() {
        // the `/api` prefixed is dropped here
        check(TestCase {
            target: "http://example.com",
            outer: "/api/abc",
            uri: "/abc",
            rewrite: RewriteKind::Nested,
            expect: "http://example.com/abc",
        });
    }

    #[test]
    fn test_into_target_uri_basic_nested_rewrite_2() {
        check(TestCase {
            target: "http://example.com/api",
            outer: "http://localhost:3000/test",
            uri: "/users/123",
            rewrite: RewriteKind::Nested,
            expect: "http://example.com/api/users/123",
        });
    }

    #[test]
    fn test_into_target_uri_basic_alias_rewrite() {
        check(TestCase {
            target: "http://example.com/api",
            outer: "http://localhost:3000/test/users/123",
            uri: "/users/123",
            rewrite: RewriteKind::Alias,
            expect: "http://example.com/api/test/users/123",
        });
    }

    #[test]
    fn test_into_target_uri_with_query_params() {
        check(TestCase {
            target: "http://example.com/api",
            outer: "http://localhost:3000/test?foo=bar",
            uri: "/users/123?foo=bar",
            rewrite: RewriteKind::Nested,
            expect: "http://example.com/api/users/123?foo=bar",
        });
    }
}
