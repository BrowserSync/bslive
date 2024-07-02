use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::inject_opts::InjectOpts;
use crate::watch_opts::WatchOpts;
use typeshare::typeshare;

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct Route {
    pub path: String,
    #[serde(flatten)]
    pub cors_opts: Option<CorsOpts>,
    #[serde(flatten)]
    pub delay_opts: Option<DelayOpts>,
    #[serde(rename = "watch")]
    #[serde(default)]
    pub watch_opts: WatchOpts,
    #[serde(rename = "inject")]
    #[serde(default)]
    pub inject_opts: InjectOpts,
    #[serde(flatten)]
    pub kind: RouteKind,
    pub headers: Option<BTreeMap<String, String>>,
}

impl Default for Route {
    fn default() -> Self {
        Self {
            path: "/".to_string(),
            kind: RouteKind::Html {
                html: "default".into(),
            },
            headers: None,
            cors_opts: None,
            delay_opts: None,
            watch_opts: Default::default(),
            inject_opts: Default::default(),
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
    Html { html: String },
    Json { json: JsonWrapper },
    Raw { raw: String },
    Sse { sse: String },
    Proxy(ProxyRoute),
    Dir(DirRoute),
}

#[derive(Debug, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub struct JsonWrapper(serde_json::Value);

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

#[typeshare]
impl RouteKind {
    #[allow(dead_code)]
    pub fn html(s: &str) -> Self {
        Self::Html { html: s.into() }
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
