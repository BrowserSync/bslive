use crate::ServerIdentityDTO;
use typeshare::typeshare;

/// @discriminator kind
#[typeshare]
#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum TransitionDTO {
    Server(ServerResourceDTO),
    PathMonitor(PathMonitorResourceDTO),
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
pub struct ServerResourceDTO {
    status: ResourceStatus,
    server: ServerIdentityDTO,
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
pub struct PathMonitorResourceDTO {
    status: ResourceStatus,
    watchable: PathWatchableDTO,
}

/// @discriminator kind
#[typeshare]
#[derive(Debug, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum PathWatchableDTO {
    Server(ServerWatchableDTO),
    Route(RouteWatchableDTO),
    Any(AnyWatchableDTO),
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
pub struct ServerWatchableDTO {
    address: String,
    dir: String,
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
pub struct RouteWatchableDTO {
    address: String,
    dir: String,
}

#[typeshare]
#[derive(Debug, serde::Serialize)]
pub struct AnyWatchableDTO {
    dir: String,
}

/// @discriminator kind
#[typeshare]
#[derive(Debug, Default, Clone, Eq, PartialEq, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ResourceStatus {
    #[default]
    Pending,
    Waiting {
        reason: String,
    },
    Ready,
    Failed {
        reason: String,
    },
}
