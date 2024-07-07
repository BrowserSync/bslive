pub mod inject_opts;
use crate::inject_opts::InjectOpts;
#[cfg(test)]
pub mod inject_opt_test;

use axum::body::Body;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::Extension;
use bytes::Bytes;
use http::header::{ACCEPT, CONTENT_LENGTH, CONTENT_TYPE};
use http::{Response, StatusCode};
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
}

#[derive(Debug, Clone)]
pub struct InjectHandling {
    pub items: InjectOpts,
}

pub async fn response_modifications_layer(
    Extension(inject): Extension<InjectHandling>,
    req: Request,
    next: Next,
) -> Result<impl IntoResponse, (StatusCode, String)> {
    let span = span!(parent: None, Level::TRACE, "resp-mod", uri=req.uri().path());
    let _guard = span.enter();
    let accepts_html = RespMod::accepts_html(&req);
    tracing::trace!(?inject);
    let mut r = next.run(req).await;
    let is_html = RespMod::is_html(&r);
    let items = inject.items.injections();
    let has_injections = !items.is_empty();

    // todo(alpha): implement named injectors, such as 'bslive:connector'
    if let (true, true, true) = (accepts_html, is_html, has_injections) {
        use http_body_util::BodyExt;
        r.headers_mut()
            .insert("x-bslive-inject", "true".parse().unwrap());
        let (mut parts, body) = r.into_parts();

        let bytes = match body.collect().await {
            Ok(collected) => collected.to_bytes(),
            Err(err) => {
                return Err((
                    StatusCode::BAD_REQUEST,
                    format!("failed to read body: {err}"),
                ))
            }
        };

        if let Ok(body) = std::str::from_utf8(&bytes) {
            tracing::trace!("replacing body content");
            let next_body = body.replace(
                "</body>",
                format!(
                    "<!-- source: snippet.html-->\
                {}\
                \
                <!-- end: snippet.html-->
                </body>",
                    include_str!("js/snippet.html")
                )
                .as_str(),
            );
            let as_bytes = Bytes::from(next_body);
            parts.headers.insert(CONTENT_LENGTH, as_bytes.len().into());
            Ok(Response::from_parts(parts, Body::from(as_bytes)))
        } else {
            tracing::trace!("could not decode from bytes");
            dbg!(&parts.headers);
            Ok((parts, bytes).into_response())
        }
    } else {
        tracing::trace!("not collecting body");
        Ok(r.into_response())
    }
}
