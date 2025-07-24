use axum::extract::Query;
use axum::response::{IntoResponse, Response};
use bsnext_resp::builtin_strings::BuiltinStrings;
use bsnext_resp::cache_opts::CacheOpts;
use http::Uri;
use std::convert::Infallible;

#[doc = include_str!("./query-params.md")]
#[derive(Debug, serde::Deserialize)]
pub struct DynamicQueryParams {
    /// Allow a request to have a ?bslive.delay.ms=200 style param to simulate a TTFB delay
    #[serde(rename = "bslive.delay.ms")]
    pub delay: Option<u64>,
    /// Control if Browsersync will add cache-busting headers, or not.
    #[serde(rename = "bslive.cache")]
    pub cache: Option<CacheOpts>,
    /// Control if Browsersync will add cache-busting headers, or not.
    #[serde(rename = "bslive.inject")]
    pub inject: Option<InjectParam>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum InjectParam {
    BuiltinStrings(BuiltinStrings),
    Other(String),
}

#[cfg(test)]
mod test {
    use super::*;
    use axum::http::Uri;
    #[test]
    fn test_deserializing() -> anyhow::Result<()> {
        let input = Uri::from_static(
            "/abc?bslive.delay.ms=2000&bslive.cache=prevent&bslive.inject=bslive:js-connector",
        );
        let Query(query_with_named): Query<DynamicQueryParams> =
            Query::try_from_uri(&input).unwrap();
        insta::assert_debug_snapshot!(query_with_named);

        let input = Uri::from_static("/abc?bslive.inject=false");
        let Query(query_with_bool): Query<DynamicQueryParams> =
            Query::try_from_uri(&input).unwrap();
        insta::assert_debug_snapshot!(query_with_bool);
        Ok(())
    }
}

pub async fn dynamic_query_params_after(uri: Uri, mut res: Response) -> impl IntoResponse {
    let Ok(Query(query_params)) = Query::try_from_uri(&uri) else {
        return Ok::<_, Infallible>(res);
    };
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
