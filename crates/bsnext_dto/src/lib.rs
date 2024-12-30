use actix::MessageResponse;
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::InputError;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::path::Path;

use crate::internal::{ServerError, StartupEvent};
use bsnext_fs::Debounce;
use bsnext_input::client_config::ClientConfig;
use bsnext_input::route::{DirRoute, ProxyRoute, RawRoute, Route, RouteKind};
use bsnext_tracing::LogLevel;
use typeshare::typeshare;

pub mod external_events;
pub mod internal;
pub mod internal_events;
pub mod startup_events;

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
            path: value.path.as_str().to_owned(),
            kind: RouteKindDTO::from(value.kind),
        }
    }
}
impl From<&Route> for RouteDTO {
    fn from(value: &Route) -> Self {
        Self {
            path: value.path.as_str().to_owned(),
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
    Dir { dir: String, base: Option<String> },
}

impl From<RouteKind> for RouteKindDTO {
    fn from(value: RouteKind) -> Self {
        match value {
            RouteKind::Raw(raw) => match raw {
                RawRoute::Html { html } => RouteKindDTO::Html { html },
                RawRoute::Json { json } => RouteKindDTO::Json {
                    json_str: serde_json::to_string(&json).expect("unreachable"),
                },
                RawRoute::Raw { raw } => RouteKindDTO::Raw { raw },
                RawRoute::Sse { sse } => RouteKindDTO::Sse { sse },
            },
            RouteKind::Proxy(ProxyRoute { proxy }) => RouteKindDTO::Proxy { proxy },
            RouteKind::Dir(DirRoute { dir, base }) => RouteKindDTO::Dir {
                dir,
                base: base.map(|b| b.to_string_lossy().to_string()),
            },
        }
    }
}

#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
pub struct ServersChangedDTO {
    pub servers_resp: GetActiveServersResponseDTO,
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
pub enum EventLevel {
    #[serde(rename = "BSLIVE_EXTERNAL")]
    External,
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum StartupEventDTO {
    Started,
    FailedStartup(String),
}

pub type StartupResult = Result<DidStart, StartupError>;

impl From<&StartupEvent> for StartupEventDTO {
    fn from(value: &StartupEvent) -> Self {
        match value {
            StartupEvent::Started => StartupEventDTO::Started,
            StartupEvent::FailedStartup(StartupError::InputError(err)) => {
                StartupEventDTO::FailedStartup(err.to_string())
            }
            StartupEvent::FailedStartup(StartupError::Other(err)) => {
                StartupEventDTO::FailedStartup(err.to_string())
            }
            StartupEvent::FailedStartup(StartupError::ServerError(err)) => {
                StartupEventDTO::FailedStartup(err.to_string())
            }
        }
    }
}

#[typeshare]
#[derive(Debug, serde::Serialize, Clone)]
pub struct InputAcceptedDTO {
    pub path: String,
}

#[typeshare]
#[derive(Debug, serde::Serialize, Clone)]
pub struct FileChangedDTO {
    pub path: String,
}

#[typeshare]
#[derive(Debug, serde::Serialize, Clone)]
pub struct FilesChangedDTO {
    pub paths: Vec<String>,
}

#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
pub struct WatchingDTO {
    pub paths: Vec<String>,
    pub debounce: DebounceDTO,
}

#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
pub struct StoppedWatchingDTO {
    pub paths: Vec<String>,
}

impl StoppedWatchingDTO {
    pub fn from_path_buf(p: &Path) -> Self {
        Self {
            paths: vec![p.to_string_lossy().to_string()],
        }
    }
}

impl Display for WatchingDTO {
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

impl FileChangedDTO {
    pub fn from_path_buf(p: &Path) -> Self {
        Self {
            path: p.to_string_lossy().to_string(),
        }
    }
}

impl WatchingDTO {
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
    pub identity: ServerIdentityDTO,
    pub change: ServerChange,
}
#[typeshare]
#[derive(Debug, serde::Serialize)]
pub struct ServerChangeSet {
    pub items: Vec<ServerChangeSetItem>,
}

#[derive(Debug, Clone, Default, MessageResponse)]
pub struct GetActiveServersResponse {
    pub servers: Vec<ActiveServer>,
}

// impl Default for GetActiveServersResponse {
//     fn default() -> Self {
//         Self { servers: vec![] }
//     }
// }

#[typeshare::typeshare]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, MessageResponse)]
pub struct GetActiveServersResponseDTO {
    pub servers: Vec<ServerDTO>,
}

impl From<&GetActiveServersResponse> for GetActiveServersResponseDTO {
    fn from(value: &GetActiveServersResponse) -> Self {
        Self {
            servers: value
                .servers
                .iter()
                .map(|s| ServerDTO {
                    id: s.identity.as_id().to_string(),
                    identity: (&s.identity).into(),
                    socket_addr: s.socket_addr.to_string(),
                })
                .collect(),
        }
    }
}

#[typeshare::typeshare]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServerDTO {
    pub id: String,
    pub identity: ServerIdentityDTO,
    pub socket_addr: String,
}

impl From<&ActiveServer> for ServerDTO {
    fn from(value: &ActiveServer) -> Self {
        Self {
            id: value.identity.as_id().to_string(),
            identity: ServerIdentityDTO::from(&value.identity),
            socket_addr: value.socket_addr.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ActiveServer {
    pub identity: ServerIdentity,
    pub socket_addr: SocketAddr,
}

#[derive(Debug)]
pub enum DidStart {
    Started(GetActiveServersResponse),
}

#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error("{0}")]
    InputError(#[from] InputError),
    #[error("{0}")]
    ServerError(#[from] ServerError),
    #[error("{0}")]
    Other(String),
}

#[typeshare::typeshare]
#[derive(Debug, PartialEq, Hash, Eq, Clone, serde::Deserialize, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ServerIdentityDTO {
    Both { name: String, bind_address: String },
    Address { bind_address: String },
    Named { name: String },
}

impl From<&ServerIdentity> for ServerIdentityDTO {
    fn from(value: &ServerIdentity) -> Self {
        match value {
            ServerIdentity::Both { name, bind_address } => ServerIdentityDTO::Both {
                name: name.to_owned(),
                bind_address: bind_address.to_owned(),
            },
            ServerIdentity::Address { bind_address } => ServerIdentityDTO::Address {
                bind_address: bind_address.to_owned(),
            },
            ServerIdentity::Named { name } => ServerIdentityDTO::Named {
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
    HtmlError(String),
    Io(String),
    UnsupportedExtension(String),
    MissingExtension(String),
    EmptyInput(String),
    BsLiveRules(String),
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
            e @ InputError::HtmlError(_) => InputErrorDTO::HtmlError(e.to_string()),
            e @ InputError::YamlError(_) => InputErrorDTO::YamlError(e.to_string()),
            e @ InputError::Io(_) => InputErrorDTO::Io(e.to_string()),
            e @ InputError::UnsupportedExtension(_) => {
                InputErrorDTO::UnsupportedExtension(e.to_string())
            }
            e @ InputError::MissingExtension(_) => InputErrorDTO::MissingExtension(e.to_string()),
            e @ InputError::EmptyInput => InputErrorDTO::EmptyInput(e.to_string()),
            e @ InputError::BsLiveRules(..) => InputErrorDTO::BsLiveRules(e.to_string()),
        }
    }
}

impl From<InputError> for InputErrorDTO {
    fn from(value: InputError) -> Self {
        (&value).into()
    }
}

#[typeshare::typeshare]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ClientEvent {
    Change(ChangeDTO),
    WsConnection(ClientConfigDTO),
    Config(ClientConfigDTO),
}

#[typeshare::typeshare]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ClientConfigDTO {
    pub log_level: LogLevelDTO,
}

impl From<ClientConfig> for ClientConfigDTO {
    fn from(value: ClientConfig) -> Self {
        Self {
            log_level: value.log.into(),
        }
    }
}

impl From<&ClientConfig> for ClientConfigDTO {
    fn from(value: &ClientConfig) -> Self {
        Self {
            log_level: value.log.into(),
        }
    }
}

#[typeshare::typeshare]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogLevelDTO {
    Info,
    Debug,
    Trace,
    Error,
}

impl From<LogLevel> for LogLevelDTO {
    fn from(value: LogLevel) -> Self {
        match value {
            LogLevel::Info => LogLevelDTO::Info,
            LogLevel::Debug => LogLevelDTO::Debug,
            LogLevel::Trace => LogLevelDTO::Trace,
            LogLevel::Error => LogLevelDTO::Error,
        }
    }
}

#[typeshare::typeshare]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ChangeDTO {
    Fs {
        path: String,
        change_kind: ChangeKind,
    },
    FsMany(Vec<ChangeDTO>),
}

#[typeshare::typeshare]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub enum ChangeKind {
    Changed,
    Added,
    Removed,
}

#[typeshare::typeshare]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct InjectConfig {
    pub connect: ConnectInfo,
    pub ctx_message: String,
}

#[typeshare::typeshare]
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ConnectInfo {
    pub ws_path: String,
    pub host: Option<String>,
}
