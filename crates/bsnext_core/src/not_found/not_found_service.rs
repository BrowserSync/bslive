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
    let span = span!(parent: None, Level::INFO, "not_found", path = req.uri().path());
    let _guard = span.enter();

    let r = next.run(req).await;
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
