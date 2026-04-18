use bsnext_input::Input;
use bsnext_input::route::Spec;
use std::path::PathBuf;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct AnyWatchable {
    pub dirs: Vec<PathBuf>,
    pub spec: Spec,
}

pub fn to_any_watchables(input: &Input) -> Vec<AnyWatchable> {
    input
        .watchers
        .iter()
        .map(|watcher| {
            let path_bufs = watcher.dirs.as_pathbufs();

            AnyWatchable {
                dirs: path_bufs,
                spec: watcher.spec.clone().unwrap_or_default(),
            }
        })
        .collect()
}
