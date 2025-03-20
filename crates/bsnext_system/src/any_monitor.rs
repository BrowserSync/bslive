use actix::Addr;
use std::hash::Hash;

use crate::input_watchable::InputWatchable;
use crate::route_watchable::RouteWatchable;
use crate::server_watchable::ServerWatchable;
use bsnext_fs::actor::FsWatcher;
use bsnext_input::route::SpecOpts;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct AnyMonitor {
    pub(crate) addr: Addr<FsWatcher>,
    pub(crate) path: PathBuf,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum AnyWatchable {
    Server(ServerWatchable),
    Input(InputWatchable),
    Route(RouteWatchable),
}

impl AnyWatchable {
    pub fn spec_opts(&self) -> Option<&SpecOpts> {
        match self {
            AnyWatchable::Server(server) => server.spec.opts.as_ref(),
            AnyWatchable::Route(route) => route.spec.opts.as_ref(),
            AnyWatchable::Input(_) => todo!("implement input spec opts"),
        }
    }
    pub fn watch_path(&self) -> &Path {
        match self {
            AnyWatchable::Server(server) => &server.dir,
            AnyWatchable::Route(route) => &route.dir,
            AnyWatchable::Input(input) => &input.path,
        }
    }
}
