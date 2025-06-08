use std::fmt::{Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::any_watchable::AnyWatchable;
use crate::route_watchable::RouteWatchable;
use crate::server_watchable::ServerWatchable;
use crate::task_list::TaskList;
use bsnext_input::route::Spec;
use std::path::Path;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum PathWatchable {
    Server(ServerWatchable),
    Route(RouteWatchable),
    Any(AnyWatchable),
}

impl Display for PathWatchable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self {
            PathWatchable::Server(server) => {
                let lines = server
                    .dirs
                    .iter()
                    .map(|x| format!("'{}'", x.display()))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "PathWatchable::Server({})", lines)?;
            }
            PathWatchable::Route(route) => {
                write!(f, "PathWatchable::Route('{}')", route.dir.display())?;
            }
            PathWatchable::Any(any) => {
                let lines = any
                    .dirs
                    .iter()
                    .map(|x| format!("'{}'", x.display()))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "PathWatchable::Any({})", lines)?;
            }
        }
        Ok(())
    }
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
