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
