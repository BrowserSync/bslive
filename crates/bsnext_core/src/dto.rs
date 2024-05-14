use actix::MessageResponse;
use bsnext_input::server_config::Identity;
use bsnext_input::InputError;
use std::fmt::{Display, Formatter};
use std::path::Path;

use crate::server::handler_change::{Change, ChangeKind};
use bsnext_fs::Debounce;
use typeshare::typeshare;

#[typeshare]
#[derive(Debug, serde::Serialize)]
pub struct ServersStarted {
    pub servers_resp: GetServersMessageResponse,
    pub changeset: ServerChangeSet,
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
#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ExternalEvents {
    ServersStarted(ServersStarted),
    Watching(Watching),
    WatchingStopped(StoppedWatching),
    FileChanged(FileChanged),
    FilesChanged(FilesChangedDTO),
    InputFileChanged(FileChanged),
    InputAccepted(InputAccepted),
    StartupFailed(InputErrorDTO),
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
pub struct InputAccepted {
    pub path: String,
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
pub struct FileChanged {
    pub path: String,
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
pub struct FilesChangedDTO {
    pub paths: Vec<String>,
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
pub struct Watching {
    pub paths: Vec<String>,
    pub debounce: DebounceDTO,
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
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
#[derive(Debug, serde::Serialize)]
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
    pub servers: Vec<ServersDTO>,
}

#[typeshare::typeshare]
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ServersDTO {
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
        }
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

impl From<&Change> for ChangeDTO {
    fn from(value: &Change) -> Self {
        match value {
            Change::Fs { path, change_kind } => Self::Fs {
                path: path.to_string_lossy().to_string(),
                change_kind: change_kind.clone(),
            },
            Change::FsMany(changes) => Self::FsMany(
                changes
                    .iter()
                    .map(|change| match change {
                        Change::Fs { path, change_kind } => Self::Fs {
                            path: path.to_string_lossy().to_string(),
                            change_kind: change_kind.clone(),
                        },
                        Change::FsMany(_) => unreachable!("recursive not supported"),
                    })
                    .collect(),
            ),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_serialize() -> anyhow::Result<()> {
        let fs = Change::fs("./a.js");
        let evt = ClientEvent::Change((&fs).into());
        let _json = serde_json::to_string(&evt).unwrap();
        Ok(())
    }
    #[test]
    fn test_serialize_server_start() -> anyhow::Result<()> {
        let fs = Change::fs("./a.js");
        let evt = ClientEvent::Change((&fs).into());
        let _json = serde_json::to_string(&evt).unwrap();
        Ok(())
    }
}
