use crate::{ExternalEventsDTO, GetServersMessageResponseDTO, StartupEventDTO};
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::startup::StartupError;
use bsnext_input::InputError;
use bsnext_output2::OutputWriterTrait;
use std::io::Write;
use std::net::SocketAddr;
use typeshare::typeshare;

#[derive(Debug)]
pub enum AnyEvent {
    Internal(InternalEvents),
    External(ExternalEventsDTO),
}
#[derive(Debug)]
pub enum InternalEvents {
    ServersChanged {
        server_resp: GetServersMessageResponseDTO,
        child_results: Vec<ChildResult>,
    },
    InputError(InputError),
    StartupError(StartupError),
}

#[derive(Debug)]
pub enum StartupEvent {
    Started,
    FailedStartup(StartupError),
}

impl OutputWriterTrait for StartupEvent {
    fn write_json<W: Write>(&self, sink: &mut W) -> anyhow::Result<()> {
        let as_dto = StartupEventDTO::from(self);
        writeln!(sink, "{}", serde_json::to_string(&as_dto)?)
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    }

    fn write_pretty<W: Write>(&self, sink: &mut W) -> anyhow::Result<()> {
        match self {
            StartupEvent::Started => {
                writeln!(sink, "started...")?;
            }
            StartupEvent::FailedStartup(err) => {
                writeln!(sink, "An error prevented startup!",)?;
                writeln!(sink)?;
                match err {
                    StartupError::InputError(InputError::BsLiveRules(bs_rules)) => {
                        let n = miette::GraphicalReportHandler::new();
                        let mut inner = String::new();
                        n.render_report(&mut inner, bs_rules).expect("write?");
                        writeln!(sink, "{}", inner)?;
                    }
                    StartupError::InputError(err) => {
                        writeln!(sink, "{}", err)?;
                    }
                }
            }
        }
        Ok(())
    }
}

/// public version of internal events
/// todo(alpha): clean this up
#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum InternalEventsDTO {
    ServersChanged(GetServersMessageResponseDTO),
}

#[derive(Debug, Clone)]
pub struct ChildHandlerMinimal {
    pub identity: ServerIdentity,
    pub socket_addr: SocketAddr,
}

#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct ChildCreated {
    pub server_handler: ChildHandlerMinimal,
}
#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct ChildPatched {
    pub server_handler: ChildHandlerMinimal,
    pub route_change_set: bsnext_input::route_manifest::RouteChangeSet,
    pub client_config_change_set: bsnext_input::client_config::ClientConfigChangeSet,
}

#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct ChildNotCreated {
    pub server_error: ServerError,
    pub identity: bsnext_input::server_config::ServerIdentity,
}

#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct ChildNotPatched {
    pub patch_error: PatchError,
    pub identity: bsnext_input::server_config::ServerIdentity,
}
#[derive(Debug)]
pub enum ChildResult {
    Created(ChildCreated),
    CreateErr(ChildNotCreated),
    Patched(ChildPatched),
    PatchErr(ChildNotPatched),
    Stopped(ServerIdentity),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, thiserror::Error)]
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

#[derive(serde::Serialize, serde::Deserialize, Debug, thiserror::Error)]
pub enum PatchError {
    // The `#[from]` attribute generates `From<JsonRejection> for ApiError`
    // implementation. See `thiserror` docs for more information
    #[error("did not patch {reason}")]
    DidNotPatch { reason: String },
}
