use crate::archy::ArchyNode;
use crate::external_events::{
    ExternalEventsDTO, InvocationIdDTO, TaskActionDTO, TaskActionStageDTO, TaskReportDTO,
    TaskResultDTO, TaskStatusDTO,
};
use crate::{GetActiveServersResponse, GetActiveServersResponseDTO, StartupError};
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::InputError;
use bsnext_task::task_report::{TaskReport, TaskResult, TaskStatus};
use std::fmt::{Display, Formatter};
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
        server_resp: GetActiveServersResponse,
        child_results: Vec<ChildResult>,
    },
    InputError(InputError),
    StartupError(StartupError),
    TaskAction(TaskAction),
}

#[derive(Debug, Clone)]
pub struct TaskReportAndTree {
    pub report: TaskReport,
    pub tree: ArchyNode,
}

#[derive(Debug, Clone)]
pub struct TaskAction {
    pub id: u64,
    pub stage: TaskActionStage,
}

#[derive(Debug, Clone)]
pub enum TaskActionStage {
    Started { tree: ArchyNode },
    Ended { tree: ArchyNode, report: TaskReport },
    Error,
}

impl TaskActionStage {
    pub fn started(id: u64, tree: ArchyNode) -> AnyEvent {
        // let action = TaskAction {
        //     id,
        //     stage: TaskActionStage::Started { tree },
        // };
        let dto = TaskActionDTO {
            id: id.to_string(),
            stage: TaskActionStageDTO::Started { tree },
        };
        AnyEvent::External(ExternalEventsDTO::TaskAction(dto))
    }
    pub fn complete(id: u64, tree: ArchyNode, report: TaskReport) -> AnyEvent {
        let dto = TaskActionDTO {
            id: id.to_string(),
            stage: TaskActionStageDTO::Ended {
                tree,
                report: TaskReportDTO::from(report),
            },
        };
        AnyEvent::External(ExternalEventsDTO::TaskAction(dto))
    }
}

impl From<TaskReport> for TaskReportDTO {
    fn from(value: TaskReport) -> Self {
        TaskReportDTO {
            id: value.id.to_string(),
            result: TaskResultDTO::from(value.result),
        }
    }
}

impl From<TaskResult> for TaskResultDTO {
    fn from(value: TaskResult) -> Self {
        TaskResultDTO {
            invocation_id: InvocationIdDTO(value.invocation_id.0.to_string()),
            status: match value.status {
                TaskStatus::Ok(_) => TaskStatusDTO::Ok,
                TaskStatus::Err(e) => TaskStatusDTO::Err(e.to_string()),
                TaskStatus::Cancelled => TaskStatusDTO::Cancelled,
            },
            task_reports: value
                .task_reports
                .into_iter()
                .map(TaskReportDTO::from)
                .collect(),
        }
    }
}

#[derive(Debug)]
pub enum StartupEvent {
    Started,
    FailedStartup(StartupError),
}

/// @discriminator kind
#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum InternalEventsDTO {
    ServersChanged(GetActiveServersResponseDTO),
    TaskReport { id: String },
}

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

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum InitialTaskError {
    #[error("initial tasks did not complete, as determined from report. TODO: access report here for better errors")]
    FailedReport,
    #[error("initial tasks did not complete, for an unknown reason")]
    FailedUnknown,
}

#[derive(Clone, serde::Serialize, serde::Deserialize, Debug, thiserror::Error)]
pub enum PatchError {
    // The `#[from]` attribute generates `From<JsonRejection> for ApiError`
    // implementation. See `thiserror` docs for more information
    #[error("did not patch {reason}")]
    DidNotPatch { reason: String },
}
