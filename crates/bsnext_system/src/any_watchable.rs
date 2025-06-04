use crate::server_watchable::{to_task_list, ServerWatchable};
use crate::task_list::TaskList;
use bsnext_input::route::Spec;
use bsnext_input::Input;
use std::path::PathBuf;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct AnyWatchable {
    pub dir: PathBuf,
    pub spec: Spec,
    pub task_list: Option<TaskList>,
}

pub fn to_any_watchables(input: &Input) -> Vec<AnyWatchable> {
    input
        .watchers
        .iter()
        .map(|watcher| {
            let runner = watcher.opts.as_ref().and_then(to_task_list);
            AnyWatchable {
                dir: PathBuf::from(&watcher.dir),
                spec: watcher.opts.clone().unwrap_or_default(),
                task_list: runner,
            }
        })
        .collect()
}
