use crate::server_watchable::to_task_list;
use crate::task_list::TaskList;
use bsnext_input::route::{DirRoute, FilterKind, RouteKind, Spec};
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::watch_opts::WatchOpts;
use bsnext_input::Input;
use std::path::PathBuf;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct RouteWatchable {
    pub server_identity: ServerIdentity,
    pub route_path: String,
    pub dir: PathBuf,
    pub spec: Spec,
    pub task_list: Option<TaskList>,
}

pub fn to_route_watchables(input: &Input) -> Vec<RouteWatchable> {
    input
        .servers
        .iter()
        .flat_map(|server_config| {
            server_config
                .routes
                .iter()
                .filter(|r| r.opts.watch.is_enabled())
                .filter_map(|r| match &r.kind {
                    RouteKind::Raw(_) => None,
                    RouteKind::Proxy(_) => None,
                    RouteKind::Dir(DirRoute { dir, .. }) => {
                        let spec = to_spec(&r.opts.watch);
                        let run = to_task_list(&spec);
                        Some(RouteWatchable {
                            server_identity: server_config.identity.clone(),
                            route_path: r.path.as_str().to_owned(),
                            dir: PathBuf::from(dir),
                            spec,
                            task_list: run,
                        })
                    }
                })
        })
        .collect()
}

pub fn to_spec(wo: &WatchOpts) -> Spec {
    match wo {
        WatchOpts::Bool(enabled) if !*enabled => unreachable!("should be handled..."),
        WatchOpts::Bool(enabled) if *enabled => Spec::default(),
        WatchOpts::InlineGlob(glob) => Spec {
            debounce: None,
            filter: Some(FilterKind::Glob {
                glob: glob.to_string(),
            }),
            ignore: None,
            run: None,
            before: None,
        },
        WatchOpts::Spec(spec) => spec.to_owned(),
        WatchOpts::Bool(_) => todo!("unreachable"),
    }
}
