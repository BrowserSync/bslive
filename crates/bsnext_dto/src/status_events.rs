use crate::ActiveServer;
use bsnext_input::server_config::ServerConfig;
use bsnext_input::Input;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, actix::Message)]
#[rtype(result = "(ServersStatus, Input)")]
pub struct ServerStatusReader;

#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct ServerStatusWriter {
    pub server_status: ServersStatus,
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
pub enum ServersStatus {
    #[default]
    Idle,
    Reconciling {
        desired: StateValue,
        observed: Option<StateValue>,
    },
    Ready {
        desired: StateValue,
        observed: StateValue,
    },
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

pub fn active_server_status_hash(servers: &[ActiveServer]) -> u64 {
    let mut identities = servers.iter().map(|x| x.content_hash).collect::<Vec<_>>();
    identities.sort_unstable();

    let mut hasher = DefaultHasher::new();
    identities.hash(&mut hasher);
    hasher.finish()
}
