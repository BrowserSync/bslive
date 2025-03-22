use bsnext_input::route::Spec;
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::Input;
use std::path::PathBuf;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct ServerWatchable {
    pub server_identity: ServerIdentity,
    pub dir: PathBuf,
    pub spec: Spec,
}

pub fn to_server_watchables(input: &Input) -> Vec<ServerWatchable> {
    input
        .servers
        .iter()
        .flat_map(|server_config| {
            server_config.watchers.iter().map(|watcher| {
                // let spec = to_spec(&r.watch_opts);
                ServerWatchable {
                    server_identity: server_config.identity.clone(),
                    dir: PathBuf::from(&watcher.dir),
                    spec: Spec {
                        opts: watcher.opts.clone(),
                    },
                }
            })
        })
        .collect()
}
