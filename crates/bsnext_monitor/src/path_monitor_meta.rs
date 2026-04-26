use crate::path_monitor::PathMonitor;
use bsnext_fs::{Debounce, FsEventContext};
use bsnext_input::route::WatchSpec;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PathMonitorMeta {
    #[allow(dead_code)]
    pub cwd: PathBuf,
    pub fs_ctx: FsEventContext,
    pub watch_spec: WatchSpec,
    pub debounce: Debounce,
}

impl From<&PathMonitor> for PathMonitorMeta {
    fn from(value: &PathMonitor) -> Self {
        Self {
            watch_spec: value.watch_spec.clone(),
            cwd: value.cwd.clone(),
            fs_ctx: value.fs_ctx,
            debounce: value.debounce,
        }
    }
}
