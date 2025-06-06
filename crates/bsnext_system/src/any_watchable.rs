use crate::server_watchable::to_task_list;
use crate::task_list::TaskList;
use bsnext_input::route::Spec;
use bsnext_input::Input;
use std::path::PathBuf;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct AnyWatchable {
    pub dirs: Vec<PathBuf>,
    pub spec: Spec,
    pub task_list: Option<TaskList>,
}

pub fn to_any_watchables(input: &Input) -> Vec<AnyWatchable> {
    input
        .watchers
        .iter()
        .map(|watcher| {
            let runner = watcher.opts.as_ref().and_then(to_task_list);
            let combined = match (&watcher.dirs, &watcher.dir) {
                (Some(dirs), Some(dir)) => {
                    let mut c = dirs.clone();
                    c.push(dir.to_string());
                    c
                }
                (Some(dirs), None) => dirs.clone(),
                (None, Some(dir)) => vec![dir.to_string()],
                _ => vec![],
            };

            AnyWatchable {
                dirs: combined.into_iter().map(PathBuf::from).collect(),
                spec: watcher.opts.clone().unwrap_or_default(),
                task_list: runner,
            }
        })
        .collect()
}
