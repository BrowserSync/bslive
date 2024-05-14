use axum::extract::{Request, State};
use axum::middleware::Next;
use std::sync::Arc;

use axum::response::{IntoResponse, Response};

use http::header::CONTENT_TYPE;
use http::{HeaderValue, StatusCode};
use mime_guess::mime;

use crate::handlers::proxy::AnyAppError;
use crate::not_found::route_list::create_route_list_html;
use crate::server::state::ServerState;
use tracing::{span, Level};

pub async fn not_found_loader(
    State(state): State<Arc<ServerState>>,
    req: Request,
    next: Next,
) -> Result<Response, AnyAppError> {
    let span = span!(parent: None, Level::INFO, "not_found", path = req.uri().path());
    let _guard = span.enter();

    let r = next.run(req).await;
    if r.status().as_u16() != 404 {
        return Ok(r);
    };

    let routes = state.routes.read().await;
    let markup = create_route_list_html(&routes);

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
