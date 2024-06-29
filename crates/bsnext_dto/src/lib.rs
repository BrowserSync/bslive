use actix::MessageResponse;
use bsnext_input::server_config::Identity;
use bsnext_input::InputError;
use std::fmt::{Display, Formatter};
use std::path::Path;

use bsnext_fs::Debounce;
use bsnext_input::route::{DirRoute, ProxyRoute, Route, RouteKind};
use bsnext_input::startup::StartupError;
use typeshare::typeshare;
pub mod internal;

#[typeshare]
#[derive(Debug, serde::Serialize)]
pub struct ServerDesc {
    pub routes: Vec<RouteDTO>,
    pub id: String,
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
pub struct RouteDTO {
    pub path: String,
    pub kind: RouteKindDTO,
}

impl From<Route> for RouteDTO {
    fn from(value: Route) -> Self {
        Self {
            path: value.path.to_owned(),
            kind: RouteKindDTO::from(value.kind),
        }
    }
}
impl From<&Route> for RouteDTO {
    fn from(value: &Route) -> Self {
        Self {
            path: value.path.to_owned(),
            kind: RouteKindDTO::from(value.kind.clone()),
        }
    }
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum RouteKindDTO {
    Html { html: String },
    Json { json_str: String },
    Raw { raw: String },
    Sse { sse: String },
    Proxy { proxy: String },
    Dir { dir: String },
}

impl From<RouteKind> for RouteKindDTO {
    fn from(value: RouteKind) -> Self {
        match value {
            RouteKind::Html { html } => RouteKindDTO::Html { html },
            RouteKind::Json { json } => RouteKindDTO::Json {
                json_str: serde_json::to_string(&json).expect("unreachable"),
            },
            RouteKind::Raw { raw } => RouteKindDTO::Raw { raw },
            RouteKind::Sse { sse } => RouteKindDTO::Sse { sse },
            RouteKind::Proxy(ProxyRoute { proxy }) => RouteKindDTO::Proxy { proxy },
            RouteKind::Dir(DirRoute { dir }) => RouteKindDTO::Dir { dir },
        }
    }
}

#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
pub struct ServersChanged {
    pub servers_resp: GetServersMessageResponse,
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
pub enum EventLevel {
    #[serde(rename = "BSLIVE_EXTERNAL")]
    External,
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
pub struct ExternalEvent {
    pub level: EventLevel,
    pub fields: ExternalEvents,
}

#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ExternalEvents {
    ServersChanged(ServersChanged),
    Watching(Watching),
    WatchingStopped(StoppedWatching),
    FileChanged(FileChanged),
    FilesChanged(FilesChangedDTO),
    InputFileChanged(FileChanged),
    InputAccepted(InputAccepted),
    InputError(InputErrorDTO),
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum StartupEvent {
    Started,
    FailedStartup(StartupErrorDTO),
}

#[typeshare]
#[derive(Debug, PartialEq, Hash, Eq, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum StartupErrorDTO {
    InputError(InputErrorDTO),
}

impl From<&StartupError> for StartupErrorDTO {
    fn from(value: &StartupError) -> Self {
        match value {
            StartupError::InputError(e) => StartupErrorDTO::InputError(e.into()),
        }
    }
}

#[typeshare]
#[derive(Debug, serde::Serialize, Clone)]
pub struct InputAccepted {
    pub path: String,
}

#[typeshare]
#[derive(Debug, serde::Serialize, Clone)]
pub struct FileChanged {
    pub path: String,
}

#[typeshare]
#[derive(Debug, serde::Serialize, Clone)]
pub struct FilesChangedDTO {
    pub paths: Vec<String>,
}

#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
pub struct Watching {
    pub paths: Vec<String>,
    pub debounce: DebounceDTO,
}

#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
pub struct StoppedWatching {
    pub paths: Vec<String>,
}

impl StoppedWatching {
    pub fn from_path_buf(p: &Path) -> Self {
        Self {
            paths: vec![p.to_string_lossy().to_string()],
        }
    }
}

impl Display for Watching {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "yo")
    }
}

#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
pub struct DebounceDTO {
    kind: String,
    ms: String,
}

impl Display for DebounceDTO {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}ms", self.kind, self.ms)
    }
}

impl From<Debounce> for DebounceDTO {
    fn from(value: Debounce) -> Self {
        match value {
            Debounce::Trailing { duration } => Self {
                kind: "trailing".to_string(),
                ms: duration.as_millis().to_string(),
            },
            Debounce::Buffered { duration } => Self {
                kind: "buffered".to_string(),
                ms: duration.as_millis().to_string(),
            },
        }
    }
}

impl FileChanged {
    pub fn from_path_buf(p: &Path) -> Self {
        Self {
            path: p.to_string_lossy().to_string(),
        }
    }
}

impl Watching {
    pub fn from_path_buf(p: &Path, debounce: Debounce) -> Self {
        Self {
            paths: vec![p.to_string_lossy().to_string()],
            debounce: debounce.into(),
        }
    }
}

#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ServerChange {
    Stopped { bind_address: String },
    Started,
    Patched,
    Errored { error: String },
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
pub struct ServerChangeSetItem {
    pub identity: IdentityDTO,
    pub change: ServerChange,
}
#[typeshare]
#[derive(Debug, serde::Serialize)]
pub struct ServerChangeSet {
    pub items: Vec<ServerChangeSetItem>,
}

#[typeshare::typeshare]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MessageResponse)]
pub struct GetServersMessageResponse {
    pub servers: Vec<ServerDTO>,
}

#[typeshare::typeshare]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerDTO {
    pub id: String,
    pub identity: IdentityDTO,
    pub socket_addr: String,
}

#[typeshare::typeshare]
#[derive(Debug, PartialEq, Hash, Eq, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum IdentityDTO {
    Both { name: String, bind_address: String },
    Address { bind_address: String },
    Named { name: String },
}

impl From<&Identity> for IdentityDTO {
    fn from(value: &Identity) -> Self {
        match value {
            Identity::Both { name, bind_address } => IdentityDTO::Both {
                name: name.to_owned(),
                bind_address: bind_address.to_owned(),
            },
            Identity::Address { bind_address } => IdentityDTO::Address {
                bind_address: bind_address.to_owned(),
            },
            Identity::Named { name } => IdentityDTO::Named {
                name: name.to_owned(),
            },
        }
    }
}

#[typeshare::typeshare]
#[derive(Debug, PartialEq, Hash, Eq, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum InputErrorDTO {
    MissingInputs(String),
    InvalidInput(String),
    NotFound(String),
    InputWriteError(String),
    PathError(String),
    PortError(String),
    DirError(String),
    YamlError(String),
    MarkdownError(String),
    Io(String),
    UnsupportedExtension(String),
    MissingExtension(String),
    EmptyInput(String),
}

impl From<&InputError> for InputErrorDTO {
    fn from(value: &InputError) -> Self {
        match value {
            e @ InputError::MissingInputs => InputErrorDTO::MissingInputs(e.to_string()),
            e @ InputError::InvalidInput(_) => InputErrorDTO::InvalidInput(e.to_string()),
            e @ InputError::NotFound(_) => InputErrorDTO::NotFound(e.to_string()),
            e @ InputError::InputWriteError(_) => InputErrorDTO::InputWriteError(e.to_string()),
            e @ InputError::PathError(_) => InputErrorDTO::PathError(e.to_string()),
            e @ InputError::PortError(_) => InputErrorDTO::PortError(e.to_string()),
            e @ InputError::DirError(_) => InputErrorDTO::DirError(e.to_string()),
            e @ InputError::MarkdownError(_) => InputErrorDTO::MarkdownError(e.to_string()),
            e @ InputError::YamlError(_) => InputErrorDTO::YamlError(e.to_string()),
            e @ InputError::Io(_) => InputErrorDTO::Io(e.to_string()),
            e @ InputError::UnsupportedExtension(_) => {
                InputErrorDTO::UnsupportedExtension(e.to_string())
            }
            e @ InputError::MissingExtension(_) => InputErrorDTO::MissingExtension(e.to_string()),
            e @ InputError::EmptyInput => InputErrorDTO::EmptyInput(e.to_string()),
        }
    }
}

impl From<InputError> for InputErrorDTO {
    fn from(value: InputError) -> Self {
        (&value).into()
    }
}

#[typeshare::typeshare]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ClientEvent {
    Change(ChangeDTO),
}

#[typeshare::typeshare]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ChangeDTO {
    Fs {
        path: String,
        change_kind: ChangeKind,
    },
    FsMany(Vec<ChangeDTO>),
}

#[typeshare::typeshare]
#[derive(Clone, Debug, serde::Serialize)]
pub enum ChangeKind {
    Changed,
    Added,
    Removed,
}
