use axum::extract::{Query, Request};
use axum::middleware::Next;
use axum::response::IntoResponse;
use std::convert::Infallible;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, serde::Deserialize)]
pub struct DynamicQueryParams {
    /// Allow a request to have a ?bslive.delay.ms=200 style param to simulate a TTFB delay
    #[serde(rename = "bslive.delay.ms")]
    delay: Option<u64>,
}

pub async fn dynamic_query_params_handler(req: Request, next: Next) -> impl IntoResponse {
    if let Ok(Query(DynamicQueryParams { delay: Some(ms) })) = Query::try_from_uri(req.uri()) {
        sleep(Duration::from_millis(ms)).await;
    }
    let res = next.run(req).await;
    Ok::<_, Infallible>(res)
}
