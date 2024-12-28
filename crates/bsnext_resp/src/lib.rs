pub mod builtin_strings;
pub mod cache_opts;
pub mod connector;
pub mod debug;
pub mod inject_addition;
pub mod inject_opts;
pub mod inject_replacement;
pub mod injector_guard;

use crate::inject_opts::InjectionItem;
#[cfg(test)]
pub mod inject_opt_test;
pub mod js_connector;

use crate::injector_guard::ByteReplacer;
use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::Extension;
use bsnext_guards::route_guard::RouteGuard;
use http::header::{ACCEPT, CONTENT_LENGTH, CONTENT_TYPE};
use http::{Response, StatusCode};
use http_body_util::BodyExt;
use tracing::{span, Level};

pub struct RespMod;

impl RespMod {
    pub fn accepts_html(req: &Request) -> bool {
        req.headers()
            .get(ACCEPT)
            .and_then(|v| v.to_str().ok().map(|s| s.contains("text/html")))
            .unwrap_or(false)
    }
    pub fn is_html<T>(res: &Response<T>) -> bool {
        res.headers()
            .get(CONTENT_TYPE)
            .and_then(|v| v.to_str().ok().map(|s| s.contains("text/html")))
            .unwrap_or(false)
    }
    pub fn is_js<T>(res: &Response<T>) -> bool {
        res.headers()
            .get(CONTENT_TYPE)
            .and_then(|v| {
                v.to_str()
                    .ok()
                    .map(|s| s.contains("application/javascript"))
            })
            .unwrap_or(false)
    }
}

#[derive(Debug, Clone)]
pub struct InjectHandling {
    pub items: Vec<InjectionItem>,
}

pub async fn response_modifications_layer(
    Extension(inject): Extension<InjectHandling>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let span = span!(parent: None, Level::TRACE, "resp-mod", uri=req.uri().path());
    let _guard = span.enter();

    // bail when there are no accepting modifications
    let req_accepted: Vec<InjectionItem> = inject
        .items
        .into_iter()
        .filter(|item| item.accept_req(&req))
        .collect();

    if req_accepted.is_empty() {
        return Ok(next.run(req).await.into_response());
    }

    let req_headers = req.headers().clone();
    let mut res = next.run(req).await;

    // also bail if no responses are accepted
    let res_accepted: Vec<InjectionItem> = req_accepted
        .into_iter()
        .filter(|item| item.accept_res(&res))
        .collect();
    if res_accepted.is_empty() {
        return Ok(res.into_response());
    }

    // if we get here, we're going to try and collect the body bytes

    let res_headers = res.headers().clone();
    res.headers_mut()
        .insert("x-bslive-inject", "true".parse().unwrap());

    let (mut parts, body) = res.into_parts();

    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(err) => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("failed to read body: {err}"),
            ))
        }
    };

    let mut next = bytes;
    for injection in &res_accepted {
        if let Some(bytes) = injection.replace_bytes(&next, &res_headers, &req_headers) {
            next = bytes
        }
    }

    parts.headers.insert(CONTENT_LENGTH, next.len().into());
    Ok(Response::from_parts(parts, Body::from(next)))
}
