use crate::tasks::task_spec::TaskSpec;
use bsnext_input::route::Spec;
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::Input;
use std::path::PathBuf;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct ServerWatchable {
    pub server_identity: ServerIdentity,
    pub dirs: Vec<PathBuf>,
    pub spec: Spec,
    pub task_spec: Option<TaskSpec>,
}

pub fn to_server_watchables(input: &Input) -> Vec<ServerWatchable> {
    input
        .servers
        .iter()
        .flat_map(|server_config| {
            server_config.watchers.iter().map(|watcher| {
                let task_spec = watcher.opts.as_ref().and_then(to_task_spec);
                let path_bufs = watcher.dirs.as_pathbufs();

                ServerWatchable {
                    server_identity: server_config.identity.clone(),
                    dirs: path_bufs,
                    spec: watcher.opts.clone().unwrap_or_default(),
                    task_spec,
                }
            })
        })
        .collect()
}

/// Convert task items into a sequential execution configuration.
/// tl;dr: Forces tasks to run in sequential order rather than concurrently.
///  
/// Creates a runner that executes tasks strictly one after another to match user
/// expectations when defining task lists in declarative formats (yaml/json).
pub fn to_task_spec(spec: &Spec) -> Option<TaskSpec> {
    // if the 'run' key was given, it's a list of steps.
    let run = spec.run.as_ref()?;

    // otherwise, construct a runner
    Some(TaskSpec::seq_from(run))
}
