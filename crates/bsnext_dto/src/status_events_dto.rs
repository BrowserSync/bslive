use crate::status_events::Transition;
use crate::{DebounceDTO, ServerIdentityDTO};
use bsnext_input::watchables::path_watchable::PathWatchable;
use typeshare::typeshare;

/// @discriminator kind
#[typeshare]
#[derive(Debug, serde::Serialize, Clone)]
#[serde(tag = "kind", content = "payload")]
pub enum TransitionDTO {
    Server(ServerResourceDTO),
    PathMonitor(PathMonitorResourceDTO),
}

impl From<Transition> for TransitionDTO {
    fn from(value: Transition) -> Self {
        match value {
            Transition::Server(server_resource) => Self::Server(ServerResourceDTO {
                server: ServerIdentityDTO::from(&server_resource.server),
                status: server_resource.status,
            }),
            Transition::PathMonitor(path_monitor_resource) => {
                Self::PathMonitor(PathMonitorResourceDTO {
                    status: path_monitor_resource.status,
                    watchable: PathWatchableDTO::from(&path_monitor_resource.watchable),
                    debounce: DebounceDTO::from(path_monitor_resource.watchable.debounce()),
                })
            }
        }
    }
}

#[typeshare]
#[derive(Debug, serde::Serialize, Clone)]
pub struct ServerResourceDTO {
    pub status: ResourceStatus,
    pub server: ServerIdentityDTO,
}

#[typeshare]
#[derive(Debug, serde::Serialize, Clone)]
pub struct PathMonitorResourceDTO {
    pub status: ResourceStatus,
    pub watchable: PathWatchableDTO,
    pub debounce: DebounceDTO,
}

/// @discriminator kind
#[typeshare]
#[derive(Debug, serde::Serialize, Clone)]
#[serde(tag = "kind", content = "payload")]
pub enum PathWatchableDTO {
    Server(ServerWatchableDTO),
    Route(RouteWatchableDTO),
    Any(AnyWatchableDTO),
    Input(InputWatchableDTO),
}

impl From<&PathWatchable> for PathWatchableDTO {
    fn from(value: &PathWatchable) -> Self {
        match value {
            PathWatchable::Server(server_watchable) => {
                let dirs = server_watchable
                    .dirs
                    .iter()
                    .map(|x| x.display().to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                Self::Server(ServerWatchableDTO {
                    dir: dirs,
                    server: ServerIdentityDTO::from(&server_watchable.server_identity),
                })
            }
            PathWatchable::Route(route_watchable) => Self::Route(RouteWatchableDTO {
                server: ServerIdentityDTO::from(&route_watchable.server_identity),
                dir: route_watchable.dir.display().to_string(),
            }),
            PathWatchable::Any(any_watchable) => {
                let dirs = any_watchable
                    .dirs
                    .iter()
                    .map(|x| x.display().to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                Self::Any(AnyWatchableDTO { dir: dirs })
            }
            PathWatchable::Input(input_watchable) => {
                let absolute_paths = input_watchable
                    .dirs
                    .iter()
                    .map(|x| x.display().to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                let relative_paths = input_watchable
                    .dirs
                    .iter()
                    .map(|x| {
                        let next = x.strip_prefix(&input_watchable.cwd).ok().unwrap_or(x);
                        next.display().to_string()
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                Self::Input(InputWatchableDTO {
                    absolute_paths,
                    relative_paths,
                })
            }
        }
    }
}

impl PathWatchableDTO {
    pub fn display_paths(&self) -> &str {
        match self {
            Self::Server(server) => server.dir.as_str(),
            Self::Route(route) => route.dir.as_str(),
            Self::Any(any) => any.dir.as_str(),
            Self::Input(input) => input.relative_paths.as_str(),
        }
    }
}

#[typeshare]
#[derive(Debug, serde::Serialize, Clone)]
pub struct ServerWatchableDTO {
    pub server: ServerIdentityDTO,
    pub dir: String,
}

#[typeshare]
#[derive(Debug, serde::Serialize, Clone)]
pub struct RouteWatchableDTO {
    pub server: ServerIdentityDTO,
    pub dir: String,
}

#[typeshare]
#[derive(Debug, serde::Serialize, Clone)]
pub struct AnyWatchableDTO {
    pub dir: String,
}

#[typeshare]
#[derive(Debug, serde::Serialize, Clone)]
pub struct InputWatchableDTO {
    pub absolute_paths: String,
    pub relative_paths: String,
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
