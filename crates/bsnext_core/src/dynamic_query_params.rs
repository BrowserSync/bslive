use axum::extract::{Query, Request};
use axum::middleware::Next;
use axum::response::IntoResponse;
use bsnext_resp::cache_opts::CacheOpts;
use std::convert::Infallible;
use std::time::Duration;
use tokio::time::sleep;

#[doc = include_str!("./query-params.md")]
#[derive(Debug, serde::Deserialize)]
pub struct DynamicQueryParams {
    /// Allow a request to have a ?bslive.delay.ms=200 style param to simulate a TTFB delay
    #[serde(rename = "bslive.delay.ms")]
    pub delay: Option<u64>,
    /// Control if Browsersync will add cache-busting headers, or not.
    #[serde(rename = "bslive.cache")]
    pub cache: Option<CacheOpts>,
}

pub async fn dynamic_query_params_handler(req: Request, next: Next) -> impl IntoResponse {
    let Ok(Query(query_params)) = Query::try_from_uri(req.uri()) else {
        let res = next.run(req).await;
        return Ok::<_, Infallible>(res);
    };

    // things to apply *before*
    #[allow(clippy::single_match)]
    match &query_params {
        DynamicQueryParams {
            delay: Some(ms), ..
        } => {
            sleep(Duration::from_millis(*ms)).await;
        }
        _ => {}
    }

    let mut res = next.run(req).await;

    // things to apply *after*
    #[allow(clippy::single_match)]
    match query_params {
        DynamicQueryParams {
            cache: Some(cache_opts),
            ..
        } => match cache_opts {
            CacheOpts::Prevent => {
                let headers_to_add = cache_opts.as_headers();
                for (name, value) in headers_to_add {
                    res.headers_mut().insert(name, value);
                }
            }
            CacheOpts::Default => {
                let headers = CacheOpts::Prevent.as_headers();
                for (name, _) in headers {
                    res.headers_mut().remove(name);
                }
            }
        },
        _ => {}
    }

    Ok::<_, Infallible>(res)
}
