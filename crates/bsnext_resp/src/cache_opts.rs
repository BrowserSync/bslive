use http::header::{CACHE_CONTROL, EXPIRES, PRAGMA};
use http::{HeaderName, HeaderValue};

#[derive(Debug, Default, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub enum CacheOpts {
    /// Try to prevent browsers from caching the responses. This is the default behaviour
    #[serde(rename = "prevent")]
    #[default]
    Prevent,
    /// Do not try to prevent browsers from caching the responses.
    #[serde(rename = "default")]
    Default,
}

impl CacheOpts {
    pub fn as_headers(&self) -> Vec<(HeaderName, HeaderValue)> {
        match self {
            CacheOpts::Prevent => {
                vec![
                    (
                        CACHE_CONTROL,
                        HeaderValue::from_static("no-store, no-cache, must-revalidate"),
                    ),
                    (PRAGMA, HeaderValue::from_static("no-cache")),
                    (EXPIRES, HeaderValue::from_static("0")),
                ]
            }
            CacheOpts::Default => {
                vec![]
            }
        }
    }
}
