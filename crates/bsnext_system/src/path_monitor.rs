use actix::Addr;
use bsnext_fs::actor::FsWatcher;
use bsnext_fs::FsEventContext;

#[derive(Debug, Clone)]
pub struct PathMonitor {
    pub(crate) addr: Addr<FsWatcher>,
    pub(crate) fs_ctx: FsEventContext,
}

impl PathMonitor {
    pub fn new(addr: Addr<FsWatcher>, fs_ctx: FsEventContext) -> Self {
        Self { addr, fs_ctx }
    }
}
