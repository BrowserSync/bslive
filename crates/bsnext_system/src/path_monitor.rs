use actix::Addr;
use bsnext_fs::actor::FsWatcher;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct PathMonitor {
    pub(crate) addr: Addr<FsWatcher>,
    pub(crate) paths: Vec<PathBuf>,
    pub(crate) watchable_hash: u64,
}
