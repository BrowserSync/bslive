use crate::task_list::TaskList;
use bsnext_input::route::Spec;
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::Input;
use std::path::PathBuf;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct ServerWatchable {
    pub server_identity: ServerIdentity,
    pub dir: PathBuf,
    pub spec: Spec,
    pub task_list: Option<TaskList>,
}

pub fn to_server_watchables(input: &Input) -> Vec<ServerWatchable> {
    input
        .servers
        .iter()
        .flat_map(|server_config| {
            server_config.watchers.iter().map(|watcher| {
                let task_list = watcher.opts.as_ref().and_then(to_task_list);

                ServerWatchable {
                    server_identity: server_config.identity.clone(),
                    dir: PathBuf::from(&watcher.dir),
                    spec: watcher.opts.clone().unwrap_or_default(),
                    task_list,
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
pub fn to_task_list(spec: &Spec) -> Option<TaskList> {
    // if the 'run' key was given, it's a list of steps.
    let run = spec.run.as_ref()?;

    // if it's empty, pretend it was absent
    if run.is_empty() {
        return None;
    };

    // otherwise, construct a runner
    Some(TaskList::seq_from(run))
}
