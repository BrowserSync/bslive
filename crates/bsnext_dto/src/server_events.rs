use bsnext_input::server_config::ServerIdentity;
use std::net::SocketAddr;

#[derive(Debug, Clone)]
pub struct ChildHandlerMinimal {
    pub identity: ServerIdentity,
    pub socket_addr: SocketAddr,
}

#[derive(Debug, Clone, actix::Message)]
#[rtype(result = "()")]
pub struct ChildCreated {
    pub server_handler: ChildHandlerMinimal,
}

#[derive(Debug, Clone, actix::Message)]
#[rtype(result = "()")]
pub struct ChildPatched {
    pub server_handler: ChildHandlerMinimal,
    pub route_change_set: bsnext_input::route_manifest::RouteChangeSet,
    pub client_config_change_set: bsnext_input::client_config::ClientConfigChangeSet,
}

#[derive(Debug, Clone, actix::Message)]
#[rtype(result = "()")]
pub struct ChildNotCreated {
    pub server_error: ServerError,
    pub identity: bsnext_input::server_config::ServerIdentity,
}

#[derive(Debug, Clone, actix::Message)]
#[rtype(result = "()")]
pub struct ChildNotPatched {
    pub patch_error: PatchError,
    pub identity: bsnext_input::server_config::ServerIdentity,
}

#[derive(Debug, Clone)]
pub enum ChildResult {
    Created(ChildCreated),
    CreateErr(ChildNotCreated),
    Patched(ChildPatched),
    PatchErr(ChildNotPatched),
    Stopped(ServerIdentity),
}

impl ChildResult {
    pub fn first_server_error(items: &[Self]) -> Option<&ServerError> {
        items
            .iter()
            .find(|x| matches!(x, ChildResult::CreateErr(..)))
            .and_then(|c| c.not_created_err())
    }
    pub fn not_created_err(&self) -> Option<&ServerError> {
        match self {
            ChildResult::CreateErr(ChildNotCreated { server_error, .. }) => Some(server_error),
            ChildResult::Created(_) => None,
            ChildResult::Patched(_) => None,
            ChildResult::PatchErr(_) => None,
            ChildResult::Stopped(_) => None,
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum ServerError {
    // The `#[from]` attribute generates `From<JsonRejection> for ApiError`
    // implementation. See `thiserror` docs for more information
    #[error("address in use {socket_addr}")]
    AddrInUse { socket_addr: SocketAddr },
    #[error("invalid bind address: {addr_parse_error}")]
    InvalidAddress { addr_parse_error: String },
    #[error("could not determine the reason: `{0}`")]
    Unknown(String),
    #[error("io error {0}")]
    Io(String),
    #[error("server was closed")]
    Closed,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, thiserror::Error)]
pub enum PatchError {
    // The `#[from]` attribute generates `From<JsonRejection> for ApiError`
    // implementation. See `thiserror` docs for more information
    #[error("did not patch {reason}")]
    DidNotPatch { reason: String },
}
