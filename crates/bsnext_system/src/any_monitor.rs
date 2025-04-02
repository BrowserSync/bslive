use std::hash::{DefaultHasher, Hash, Hasher};

use crate::path_monitor::PathMonitor;
use crate::route_watchable::RouteWatchable;
use crate::runner::Runner;
use crate::server_watchable::ServerWatchable;
use actix::Addr;
use bsnext_fs::actor::FsWatcher;
use bsnext_input::route::SpecOpts;
use std::path::Path;

#[derive(Debug, Clone)]
pub enum AnyMonitor {
    Path(PathMonitor),
}

impl AnyMonitor {
    pub fn fs_addr(&self) -> &Addr<FsWatcher> {
        match self {
            AnyMonitor::Path(path) => &path.addr,
        }
    }

    pub fn watchable_hash(&self) -> u64 {
        match self {
            AnyMonitor::Path(path) => path.watchable_hash,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum PathWatchable {
    Server(ServerWatchable),
    Route(RouteWatchable),
}

impl PathWatchable {
    pub fn spec_opts(&self) -> Option<&SpecOpts> {
        match self {
            PathWatchable::Server(server) => server.spec.opts.as_ref(),
            PathWatchable::Route(route) => route.spec.opts.as_ref(),
        }
    }
    pub fn watch_path(&self) -> &Path {
        match self {
            PathWatchable::Server(server) => &server.dir,
            PathWatchable::Route(route) => &route.dir,
        }
    }

    pub fn as_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn runner(&self) -> Option<&Runner> {
        match self {
            PathWatchable::Server(server) => server.runner.as_ref(),
            PathWatchable::Route(route) => route.runner.as_ref(),
        }
    }
}
