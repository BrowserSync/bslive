use axum::extract::{Query, Request};
use axum::middleware::Next;
use axum::response::IntoResponse;
use bsnext_guards::path_matcher::PathMatcher;
use bsnext_guards::MatcherList;
use bsnext_resp::builtin_strings::{BuiltinStringDef, BuiltinStrings};
use bsnext_resp::cache_opts::CacheOpts;
use bsnext_resp::inject_opts::{Injection, InjectionItem};
use bsnext_resp::InjectHandling;
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
    use http::Uri;
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

pub async fn dynamic_query_params_handler(mut req: Request, next: Next) -> impl IntoResponse {
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

    // Other things to apply *before*
    #[allow(clippy::single_match)]
    match &query_params {
        DynamicQueryParams {
            inject: Some(inject_append),
            ..
        } => {
            let uri = req.uri().clone();
            let ex = req.extensions_mut();
            if let Some(inject) = ex.get_mut::<InjectHandling>() {
                tracing::info!(
                    "Adding an item to the injection handling on the fly {} {}",
                    uri.to_string(),
                    uri.path()
                );
                match inject_append {
                    InjectParam::Other(other) if other == "false" => {
                        inject.items = vec![];
                    }
                    InjectParam::BuiltinStrings(str) => inject.items.push(InjectionItem {
                        inner: Injection::BsLive(BuiltinStringDef {
                            name: str.to_owned(),
                        }),
                        only: Some(MatcherList::Item(PathMatcher::pathname(uri.path()))),
                    }),
                    InjectParam::Other(_) => todo!("other?"),
                }
            }
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
