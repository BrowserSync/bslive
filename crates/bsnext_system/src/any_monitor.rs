use std::hash::{DefaultHasher, Hash, Hasher};

use crate::any_watchable::AnyWatchable;
use crate::path_monitor::PathMonitor;
use crate::route_watchable::RouteWatchable;
use crate::server_watchable::ServerWatchable;
use crate::task_list::TaskList;
use actix::Addr;
use bsnext_fs::actor::FsWatcher;
use bsnext_input::route::Spec;
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
    Any(AnyWatchable),
}

impl PathWatchable {
    pub fn spec_opts(&self) -> &Spec {
        match self {
            PathWatchable::Server(server) => &server.spec,
            PathWatchable::Route(route) => &route.spec,
            PathWatchable::Any(any) => &any.spec,
        }
    }
    pub fn watch_paths(&self) -> Vec<&Path> {
        match self {
            PathWatchable::Server(server) => server.dirs.iter().map(|x| x.as_path()).collect(),
            PathWatchable::Route(route) => vec![&route.dir],
            PathWatchable::Any(any) => any.dirs.iter().map(|x| x.as_path()).collect(),
        }
    }

    pub fn as_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn task_list(&self) -> Option<&TaskList> {
        match self {
            PathWatchable::Server(server) => server.task_list.as_ref(),
            PathWatchable::Route(route) => route.task_list.as_ref(),
            PathWatchable::Any(any) => any.task_list.as_ref(),
        }
    }
}
