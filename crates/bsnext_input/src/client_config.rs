use bsnext_tracing::LogLevel;
use std::cmp::PartialEq;

#[derive(Debug, Default, PartialEq, Clone, serde::Deserialize, serde::Serialize)]
pub struct ClientConfig {
    pub log: LogLevel,
}

impl ClientConfig {
    pub fn changeset_for(&self, p0: &ClientConfig) -> ClientConfigChangeSet {
        if self == p0 {
            ClientConfigChangeSet { changed: vec![] }
        } else {
            ClientConfigChangeSet {
                changed: vec![p0.clone()],
            }
        }
    }
}

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct ClientConfigChangeSet {
    pub changed: Vec<ClientConfig>,
}
