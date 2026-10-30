use crate::status_events_dto::ResourceStatus;
use crate::ActiveServer;
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use bsnext_input::watchables::path_watchable::PathWatchable;
use bsnext_input::watchables::PathWatchables;
use bsnext_input::Input;
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, actix::Message)]
#[rtype(result = "(ServiceStatus, Input)")]
pub struct ServerStatusReader;

#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct ServerStatusWriter {
    pub servers_status: ServiceStatus,
}

#[derive(Debug, actix::Message)]
#[rtype(result = "(ServiceStatus, Input)")]
pub struct MonitorStatusReader;

#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct MonitorStatusWriter {
    pub monitor_status: ServiceStatus,
    pub monitor_resource_status: HashMap<PathWatchable, ResourceStatus>,
}

/// This is the event we'll broadcast, it will grow with more sub systems
#[derive(Debug)]
pub enum Transition {
    Server(ServerResource),
    PathMonitor(PathMonitorResource),
}

impl Transition {
    pub fn path_monitor(status: ResourceStatus, watchable: PathWatchable) -> Self {
        Transition::PathMonitor(PathMonitorResource::new(status, watchable))
    }
}

#[derive(Debug)]
pub struct ServerResource {
    pub status: ResourceStatus,
    pub server: ServerIdentity,
}

#[derive(Debug)]
pub struct PathMonitorResource {
    pub status: ResourceStatus,
    pub watchable: PathWatchable,
}

impl PathMonitorResource {
    pub fn new(status: ResourceStatus, watchable: PathWatchable) -> Self {
        Self { status, watchable }
    }
}

#[derive(Debug, Clone, Hash, Copy, Eq, Ord, PartialEq, PartialOrd)]
pub struct StateValue {
    value: u64,
}

impl StateValue {
    pub fn new(value: u64) -> Self {
        Self { value }
    }
}

#[derive(Debug, Default, Clone, Hash, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum ServiceStatus {
    #[default]
    Idle,
    Reconciling {
        desired: StateValue,
        observed: Option<StateValue>,
    },
    Ready {
        observed: StateValue,
    },
}

impl ServiceStatus {
    pub fn observed(&self) -> Option<StateValue> {
        match self {
            ServiceStatus::Idle => None,
            ServiceStatus::Reconciling { observed, .. } => *observed,
            ServiceStatus::Ready { observed, .. } => Some(*observed),
        }
    }
}

pub fn server_status_hash(servers: &[ServerConfig]) -> u64 {
    let mut identities = servers
        .iter()
        .map(|x| {
            let mut hasher = DefaultHasher::new();
            x.hash(&mut hasher);
            hasher.finish()
        })
        .collect::<Vec<_>>();
    identities.sort_unstable();

    let mut hasher = DefaultHasher::new();
    identities.hash(&mut hasher);
    hasher.finish()
}

pub fn monitor_status_hash(path_watchables: &PathWatchables) -> u64 {
    let mut identities = path_watchables
        .watchables
        .iter()
        .map(|x| {
            let mut hasher = DefaultHasher::new();
            x.hash(&mut hasher);
            hasher.finish()
        })
        .collect::<Vec<_>>();
    identities.sort_unstable();

    let mut hasher = DefaultHasher::new();
    identities.hash(&mut hasher);
    hasher.finish()
}

pub fn list_hash(items: &[u64]) -> u64 {
    let mut identities = items.iter().collect::<Vec<_>>();
    identities.sort_unstable();

    let mut hasher = DefaultHasher::new();
    identities.hash(&mut hasher);
    hasher.finish()
}

pub fn active_server_status_hash(servers: &[ActiveServer]) -> u64 {
    let mut identities = servers.iter().map(|x| x.content_hash).collect::<Vec<_>>();
    identities.sort_unstable();

    let mut hasher = DefaultHasher::new();
    identities.hash(&mut hasher);
    hasher.finish()
}
