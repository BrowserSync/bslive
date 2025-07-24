use axum::extract::Query;
use axum::response::IntoResponse;
use bsnext_resp::builtin_strings::BuiltinStrings;
use bsnext_resp::cache_opts::CacheOpts;
use http::Uri;

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
