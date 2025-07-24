use crate::path_def::PathDef;
use crate::route_cli::RouteCli;
use crate::watch_opts::WatchOpts;
use crate::when_guard::{WhenBodyGuard, WhenGuard};
use bsnext_resp::cache_opts::CacheOpts;
use bsnext_resp::inject_opts::InjectOpts;
use matchit::InsertError;
use std::collections::BTreeMap;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct Route {
    #[serde(with = "crate::path_def")]
    pub path: PathDef,
    #[serde(flatten)]
    pub kind: RouteKind,
    #[serde(flatten)]
    pub opts: Opts,
    pub fallback: Option<FallbackRoute>,
    pub when: Option<ListOrSingle<WhenGuard>>,
    pub when_body: Option<ListOrSingle<WhenBodyGuard>>,
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum ListOrSingle<T> {
    WhenOne(T),
    WhenMany(Vec<T>),
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum PathDefError {
    #[error("Paths must start with a slash")]
    StartsWithSlash,
    #[error("Paths cannot contain a `*`")]
    ContainsStar,
    #[error("matchit error {0}")]
    InsertError(#[from] InsertError),
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct FallbackRoute {
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
    #[serde(default)]
    pub cache: CacheOpts,
    pub compression: Option<CompressionOpts>,
}

impl Default for Route {
    fn default() -> Self {
        Self {
            path: PathDef::from_str("/").unwrap(),
            kind: RouteKind::new_html("default"),
            opts: Opts {
                ..Default::default()
            },
            fallback: Default::default(),
            when: Default::default(),
            when_body: None,
        }
    }
}

impl AsRef<Route> for Route {
    fn as_ref(&self) -> &Route {
        self
    }
}

impl Route {
    pub fn url_path(&self) -> &str {
        self.path.as_str()
    }
    pub fn path_buf(&self) -> PathBuf {
        PathBuf::from(self.path.as_str())
    }
    pub fn as_filepath(&self) -> PathBuf {
        let next = PathBuf::from(self.path.as_str());

        let next = if next == PathBuf::from("/") {
            next.join("index.html")
        } else {
            next
        };

        if next.starts_with("/") {
            next.strip_prefix("/").unwrap().to_path_buf()
        } else {
            next
        }
    }
    pub fn from_cli_str<A: AsRef<str>>(a: A) -> Result<Self, anyhow::Error> {
        let cli = RouteCli::try_from_cli_str(a)?;
        cli.try_into()
    }

    pub fn proxy<A: AsRef<str>>(a: A) -> Self {
        Self {
            path: PathDef::root(),
            opts: Opts {
                cors: Some(CorsOpts::Cors(true)),
                ..Default::default()
            },
            kind: RouteKind::Proxy(ProxyRoute {
                proxy: a.as_ref().to_string(),
                proxy_headers: None,
                rewrite_uri: None,
                unstable_mirror: None,
            }),
            ..Default::default()
        }
    }
}

#[derive(Debug, Hash, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum RouteKind {
    Raw(RawRoute),
    Proxy(ProxyRoute),
    Dir(DirRoute),
}

impl Display for RouteKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            RouteKind::Raw(raw) => match raw {
                RawRoute::Html { html: _ } => write!(f, "Raw(HTML)"),
                RawRoute::Json { json: _ } => write!(f, "Raw(JSON)"),
                RawRoute::Raw { raw: _ } => write!(f, "Raw(Text)"),
                RawRoute::Sse { sse: _ } => write!(f, "Raw(SSE)"),
            },
            RouteKind::Proxy(proxy) => write!(f, "Proxy({})", proxy.proxy),
            RouteKind::Dir(dir) => write!(f, "Dir({})", dir.dir),
        }
    }
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
        RouteKind::Raw(RawRoute::Sse {
            sse: SseOpts {
                body: raw.into(),
                throttle_ms: None,
            },
        })
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

#[derive(Debug, Default, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct DirRoute {
    pub dir: String,
    pub base: Option<PathBuf>,
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct ProxyRoute {
    pub proxy: String,
    pub proxy_headers: Option<BTreeMap<String, String>>,
    pub rewrite_uri: Option<bool>,
    pub unstable_mirror: Option<String>,
}

impl ProxyRoute {
    pub fn mirror(&self) -> Option<PathBuf> {
        self.unstable_mirror.as_ref().map(PathBuf::from)
    }
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
struct Mirror {
    pub dir: String,
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum RawRoute {
    Html { html: String },
    Json { json: JsonWrapper },
    Raw { raw: String },
    Sse { sse: SseOpts },
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct SseOpts {
    pub body: String,
    pub throttle_ms: Option<u64>,
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub enum CorsOpts {
    #[serde(rename = "cors")]
    Cors(bool),
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum CompressionOpts {
    Bool(bool),
    CompType(CompType),
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub enum CompType {
    #[serde(rename = "gzip")]
    Gzip,
    #[serde(rename = "br")]
    Br,
    #[serde(rename = "deflate")]
    Deflate,
    #[serde(rename = "zstd")]
    Zstd,
}

impl Default for CompressionOpts {
    fn default() -> Self {
        Self::Bool(false)
    }
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
    StringDefault(String),
    Extension { ext: String },
    Glob { glob: String },
    Any { any: String },
    List(Vec<FilterKind>),
}

#[derive(
    Debug,
    Default,
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Hash,
    Clone,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct Spec {
    pub debounce: Option<DebounceDuration>,
    pub filter: Option<FilterKind>,
    pub ignore: Option<FilterKind>,
    pub run: Option<Vec<RunOptItem>>,
    pub before: Option<Vec<BeforeRunOptItem>>,
}

#[derive(
    Debug,
    Default,
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Hash,
    Clone,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct ShRunOptItem {
    pub sh: String,
    pub name: Option<String>,
    pub prefix: Option<PrefixOpt>,
}

impl ShRunOptItem {
    pub fn new(str: &str) -> Self {
        ShRunOptItem {
            sh: str.to_string(),
            ..std::default::Default::default()
        }
    }
}

#[derive(
    Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, serde::Deserialize, serde::Serialize,
)]
#[serde(untagged)]
pub enum RunOptItem {
    BsLive { bslive: BsLiveRunner },
    Sh(ShRunOptItem),
    All(RunAll),
    Seq(RunSeq),
    ShImplicit(String),
}
#[derive(
    Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, serde::Deserialize, serde::Serialize,
)]
#[serde(untagged)]
pub enum BeforeRunOptItem {
    Sh(ShRunOptItem),
    All(RunAll),
    Seq(RunSeq),
}

impl BeforeRunOptItem {
    pub fn into_run_opt(self) -> RunOptItem {
        match self {
            BeforeRunOptItem::Sh(sh) => RunOptItem::Sh(sh),
            BeforeRunOptItem::All(run_all) => RunOptItem::All(run_all),
            BeforeRunOptItem::Seq(run_seq) => RunOptItem::Seq(run_seq),
        }
    }
}

#[derive(
    Debug,
    Default,
    Ord,
    PartialOrd,
    PartialEq,
    Eq,
    Hash,
    Clone,
    serde::Deserialize,
    serde::Serialize,
)]
pub struct RunAll {
    pub all: Vec<RunOptItem>,
    #[serde(default, rename = "opts")]
    pub run_all_opts: RunAllOpts,
}

impl RunAll {
    pub fn new(items: Vec<RunOptItem>) -> Self {
        Self {
            all: items,
            run_all_opts: Default::default(),
        }
    }

    pub fn with_opts(items: Vec<RunOptItem>, opts: RunAllOpts) -> Self {
        Self {
            all: items,
            run_all_opts: opts,
        }
    }

    pub fn items(&self) -> &Vec<RunOptItem> {
        &self.all
    }
}

#[derive(
    Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, serde::Deserialize, serde::Serialize,
)]
pub struct RunAllOpts {
    pub max: u8,
}

impl Default for RunAllOpts {
    fn default() -> Self {
        Self { max: 5 }
    }
}

#[derive(
    Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, serde::Deserialize, serde::Serialize,
)]
pub struct SeqOpts {
    pub exit_on_fail: bool,
}

impl Default for SeqOpts {
    fn default() -> Self {
        Self { exit_on_fail: true }
    }
}

#[derive(
    Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, serde::Deserialize, serde::Serialize,
)]
pub struct RunSeq {
    pub seq: Vec<RunOptItem>,
    #[serde(default, rename = "opts")]
    pub seq_opts: SeqOpts,
}

impl RunSeq {
    pub fn new(items: Vec<RunOptItem>) -> Self {
        Self {
            seq: items,
            seq_opts: std::default::Default::default(),
        }
    }
    pub fn with_opts(items: Vec<RunOptItem>, seq_opts: SeqOpts) -> Self {
        Self {
            seq: items,
            seq_opts,
        }
    }

    pub fn items(&self) -> &Vec<RunOptItem> {
        &self.seq
    }
}

#[derive(
    Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, serde::Deserialize, serde::Serialize,
)]
#[serde(untagged)]
pub enum PrefixOpt {
    Bool(bool),
    Named(String),
}

impl Default for PrefixOpt {
    fn default() -> Self {
        Self::Bool(true)
    }
}

#[derive(
    Debug, Ord, PartialOrd, PartialEq, Eq, Hash, Clone, serde::Deserialize, serde::Serialize,
)]
pub enum BsLiveRunner {
    #[serde(rename = "notify-server")]
    NotifyServer,
    #[serde(rename = "ext-event")]
    ExtEvent,
}

impl Display for BsLiveRunner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BsLiveRunner::NotifyServer => write!(f, "BsLiveRunner::NotifyServer"),
            BsLiveRunner::ExtEvent => write!(f, "BsLiveRunner::ExtEvent"),
        }
    }
}

#[derive(
    Debug, PartialOrd, Ord, Eq, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize,
)]
pub struct MultiWatch {
    pub dirs: WatcherDirs,
    #[serde(flatten)]
    pub opts: Option<Spec>,
}

#[derive(
    Debug, PartialOrd, Ord, Eq, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize,
)]
#[serde(untagged)]
pub enum WatcherDirs {
    Single(String),
    Many(Vec<String>),
}

impl WatcherDirs {
    pub fn one(path_str: &str) -> Self {
        Self::Single(path_str.to_string())
    }
    pub fn many(path_str: &[&str]) -> Self {
        Self::Many(path_str.iter().map(ToString::to_string).collect())
    }
    pub fn first_as_pathbuf(&self) -> PathBuf {
        match self {
            WatcherDirs::Single(item) => PathBuf::from(item),
            WatcherDirs::Many(item) if !item.is_empty() => {
                PathBuf::from(item.first().expect("guarded"))
            }
            WatcherDirs::Many(_) => todo!("cannot get here?"),
        }
    }
    pub fn as_pathbufs(&self) -> Vec<PathBuf> {
        match self {
            WatcherDirs::Single(item) => vec![PathBuf::from(item)],
            WatcherDirs::Many(item) if !item.is_empty() => item.iter().map(PathBuf::from).collect(),
            WatcherDirs::Many(_) => vec![],
        }
    }
}

impl MultiWatch {
    pub fn add_task(&mut self, item: RunOptItem) {
        let opts = self.opts.get_or_insert_default();
        let list = opts.run.get_or_insert(vec![]);
        list.push(item);
    }
}
