use bsnext_input::Input;
use bsnext_input::route::Spec;
use bsnext_input::server_config::ServerIdentity;
use std::path::PathBuf;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct ServerWatchable {
    pub server_identity: ServerIdentity,
    pub dirs: Vec<PathBuf>,
    pub spec: Spec,
}

pub fn to_server_watchables(input: &Input) -> Vec<ServerWatchable> {
    input
        .servers
        .iter()
        .flat_map(|server_config| {
            server_config.watchers.iter().map(|watcher| {
                let path_bufs = watcher.dirs.as_pathbufs();

                ServerWatchable {
                    server_identity: server_config.identity.clone(),
                    dirs: path_bufs,
                    spec: watcher.spec.clone().unwrap_or_default(),
                }
            })
        })
        .collect()
}
