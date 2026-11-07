use std::fmt::{Debug, Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};

use crate::route::WatchSpec;
use crate::watchables::any_watchable::AnyWatchable;
use crate::watchables::input_watchable::InputWatchable;
use crate::watchables::route_watchable::RouteWatchable;
use crate::watchables::server_watchable::ServerWatchable;
use bsnext_fs::Debounce;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum PathWatchable {
    Server(ServerWatchable),
    Route(RouteWatchable),
    Any(AnyWatchable),
    Input(InputWatchable),
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
                write!(f, "PathWatchable::Server({lines})")?;
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
                write!(f, "PathWatchable::Any({lines})")?;
            }
            PathWatchable::Input(input_watchable) => {
                let lines = input_watchable
                    .dirs
                    .iter()
                    .map(|x| format!("'{}'", x.display()))
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "PathWatchable::Input({lines})")?;
            }
        }
        Ok(())
    }
}

impl PathWatchable {
    pub fn spec(&self) -> &WatchSpec {
        match self {
            PathWatchable::Server(server) => &server.watch_spec,
            PathWatchable::Route(route) => &route.watch_spec,
            PathWatchable::Any(any) => &any.watch_spec,
            PathWatchable::Input(input) => &input.watch_spec,
        }
    }
    pub fn watch_paths(&self) -> Vec<&Path> {
        match self {
            PathWatchable::Server(server) => server.dirs.iter().map(|x| x.as_path()).collect(),
            PathWatchable::Route(route) => vec![&route.dir],
            PathWatchable::Any(any) => any.dirs.iter().map(|x| x.as_path()).collect(),
            PathWatchable::Input(input) => input.dirs.iter().map(|x| x.as_path()).collect(),
        }
    }

    pub fn as_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn debounce(&self) -> Debounce {
        let spec = match self {
            PathWatchable::Server(server) => Some(&server.watch_spec),
            PathWatchable::Route(route) => Some(&route.watch_spec),
            PathWatchable::Any(any) => Some(&any.watch_spec),
            PathWatchable::Input(_input) => None,
        };
        match spec {
            None => Debounce::Trailing {
                duration: Duration::from_millis(300),
            },
            Some(spec) => spec.debounce.map(Debounce::from).unwrap_or_default(),
        }
    }
}
