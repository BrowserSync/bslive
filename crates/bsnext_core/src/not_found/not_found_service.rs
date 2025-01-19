use axum::extract::Request;
use axum::middleware::Next;

use axum::response::{IntoResponse, Response};

use http::header::CONTENT_TYPE;
use http::{HeaderValue, StatusCode};
use mime_guess::mime;

use crate::handlers::proxy::AnyAppError;

use bsnext_client::html_with_base;
use tracing::{span, Level};

pub async fn not_found_loader(req: Request, next: Next) -> Result<Response, AnyAppError> {
    let span = span!(parent: None, Level::TRACE, "not_found_mw", path = req.uri().path());
    let _guard = span.enter();

    tracing::trace!("not_found->");
    let r = next.run(req).await;
    tracing::trace!("<-not_found");
    if r.status().as_u16() != 404 {
        return Ok(r);
    };

    let markup = html_with_base("/__bs_assets/ui/");

    Ok((
        StatusCode::NOT_FOUND,
        [(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML_UTF_8.as_ref()),
        )],
        markup,
    )
        .into_response())
}

pub async fn not_found_srv(req: Request) -> Result<Response, AnyAppError> {
    let span = span!(parent: None, Level::TRACE, "not_found_srv", path = req.uri().path());
    let _guard = span.enter();
    tracing::trace!("responding");

    let markup = html_with_base("/__bs_assets/ui/");

    Ok((
        StatusCode::NOT_FOUND,
        [(
            CONTENT_TYPE,
            HeaderValue::from_static(mime::TEXT_HTML_UTF_8.as_ref()),
        )],
        markup,
    )
        .into_response())
}

pub async fn maybe_proxy(req: Request, next: Next) -> impl IntoResponse {
    let span = span!(parent: None, Level::INFO, "maybe_proxy", path = req.uri().path());
    let _guard = span.enter();

    tracing::trace!("p->");
    let r = next.run(req).await;
    tracing::trace!("<-p");
    if r.status().as_u16() != 404 {
        return r.into_response();
    };

    "what can I do?".into_response()
}
