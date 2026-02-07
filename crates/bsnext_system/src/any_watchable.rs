use crate::server_watchable::to_task_spec;
use crate::tasks::task_spec::TaskSpec;
use bsnext_input::route::Spec;
use bsnext_input::Input;
use std::path::PathBuf;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct AnyWatchable {
    pub dirs: Vec<PathBuf>,
    pub spec: Spec,
    pub task_spec: Option<TaskSpec>,
}

pub fn to_any_watchables(input: &Input) -> Vec<AnyWatchable> {
    input
        .watchers
        .iter()
        .map(|watcher| {
            let task_spec = watcher.opts.as_ref().and_then(to_task_spec);
            let path_bufs = watcher.dirs.as_pathbufs();

            AnyWatchable {
                dirs: path_bufs,
                spec: watcher.opts.clone().unwrap_or_default(),
                task_spec,
            }
        })
        .collect()
}
