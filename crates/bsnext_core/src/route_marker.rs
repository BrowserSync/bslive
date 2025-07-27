use crate::server::router::ProxyResponseEncoding;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use http::header::CONTENT_ENCODING;
use http::{HeaderName, HeaderValue};
use tracing::trace;

#[derive(Debug, Clone)]
pub struct RouteMarker;

impl RouteMarker {
    pub async fn after_decompress(req: Request, next: Next) -> impl IntoResponse {
        let mut res = next.run(req).await;
        if let Some((k, v)) = match (
            res.extensions().get::<ProxyResponseEncoding>(),
            res.headers().get(CONTENT_ENCODING),
        ) {
            // if the original CONTENT_ENCODING header was removed
            (Some(original), None) => {
                let bytes = original.0.as_bytes();
                Some((
                    HeaderName::from_static("x-bslive-decompressed-from"),
                    HeaderValue::from_bytes(bytes).unwrap_or(HeaderValue::from_static("unknown")),
                ))
            }
            _ => None,
        } {
            res.headers_mut().insert(k, v);
        }
        res
    }

    pub async fn before_proxy(req: Request, next: Next) -> impl IntoResponse {
        let mut res = next.run(req).await;
        let prev = {
            res.headers()
                .get(CONTENT_ENCODING)
                .as_ref()
                .map(|v| (*v).to_owned())
        };
        if let Some(hv) = prev {
            trace!(?hv, "recording a content-encoding header");
            res.extensions_mut()
                .insert(ProxyResponseEncoding(hv.to_str().unwrap().to_string()));
        }
        res
    }
}
