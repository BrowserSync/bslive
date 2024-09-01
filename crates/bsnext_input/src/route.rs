use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::watch_opts::WatchOpts;
use bsnext_resp::inject_opts::InjectOpts;

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct Route {
    pub path: String,
    #[serde(flatten)]
    pub kind: RouteKind,
    #[serde(flatten)]
    pub opts: Opts,
}

#[derive(Debug, Default, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct Opts {
    #[serde(flatten)]
    pub cors: Option<CorsOpts>,
    #[serde(flatten)]
    pub delay: Option<DelayOpts>,
    #[serde(default)]
    pub watch: WatchOpts,
    #[serde(default)]
    pub inject: InjectOpts,
    pub headers: Option<BTreeMap<String, String>>,
}

impl Default for Route {
    fn default() -> Self {
        Self {
            path: "/".to_string(),
            kind: RouteKind::new_html("default"),
            opts: Opts {
                headers: None,
                cors: None,
                delay: None,
                watch: Default::default(),
                inject: Default::default(),
            },
        }
    }
}

impl AsRef<Route> for Route {
    fn as_ref(&self) -> &Route {
        self
    }
}

impl Route {
    pub fn path(&self) -> &str {
        self.path.as_str()
    }
}

#[derive(Debug, Hash, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum RouteKind {
    Raw(RawRoute),
    Proxy(ProxyRoute),
    Dir(DirRoute),
}

impl RouteKind {
    pub fn new_html(html: impl Into<String>) -> Self {
        RouteKind::Raw(RawRoute::Html { html: html.into() })
    }
    pub fn new_raw(raw: impl Into<String>) -> Self {
        RouteKind::Raw(RawRoute::Raw { raw: raw.into() })
    }
    pub fn new_json(json: JsonWrapper) -> Self {
        RouteKind::Raw(RawRoute::Json { json })
    }
    pub fn new_sse(raw: impl Into<String>) -> Self {
        RouteKind::Raw(RawRoute::Sse { sse: raw.into() })
    }
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub struct JsonWrapper(pub serde_json::Value);

impl Display for JsonWrapper {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self.0).expect("serde_json"))
    }
}

impl Deref for JsonWrapper {
    type Target = serde_json::Value;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Hash for JsonWrapper {
    fn hash<H: Hasher>(&self, state: &mut H) {
        //todo: implement hashing for serde_value
        if let Ok(as_str) = serde_json::to_string(&self.0) {
            state.write(as_str.as_bytes());
        } else {
            todo!("handle error here?")
        }
    }
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct DirRoute {
    pub dir: String,
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct ProxyRoute {
    pub proxy: String,
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum RawRoute {
    Html { html: String },
    Json { json: JsonWrapper },
    Raw { raw: String },
    Sse { sse: String },
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub enum CorsOpts {
    #[serde(rename = "cors")]
    Cors(bool),
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub enum DelayOpts {
    #[serde(rename = "delay")]
    Delay(DelayKind),
}

#[derive(
    Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, serde::Deserialize, serde::Serialize,
)]
pub enum DelayKind {
    #[serde(rename = "ms")]
    Ms(u64),
}

#[derive(
    Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, serde::Deserialize, serde::Serialize,
)]
pub enum DebounceDuration {
    #[serde(rename = "ms")]
    Ms(u64),
}

#[derive(
    Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, serde::Deserialize, serde::Serialize,
)]
#[serde(untagged)]
pub enum FilterKind {
    StringGlob(String),
    Extension { ext: String },
    Glob { glob: String },
    List(Vec<FilterKind>),
}

#[derive(
    Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, serde::Deserialize, serde::Serialize,
)]
pub struct Spec {
    #[serde(flatten)]
    pub opts: Option<SpecOpts>,
}

#[derive(
    Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, serde::Deserialize, serde::Serialize,
)]
pub struct SpecOpts {
    pub debounce: Option<DebounceDuration>,
    pub filter: Option<FilterKind>,
}

#[derive(
    Debug, PartialOrd, Ord, Eq, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize,
)]
pub struct Watcher {
    pub dir: String,
    pub debounce_ms: Option<u64>,
    pub filter: Option<FilterKind>,
}
