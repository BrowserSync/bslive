use crate::server_watchable::to_task_list;
use crate::tasks::task_list::TaskList;
use bsnext_input::route::{DirRoute, FilterKind, RawRoute, RouteKind, Spec, SseOpts};
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::watch_opts::WatchOpts;
use bsnext_input::Input;
use std::path::{Path, PathBuf};

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
                .filter_map(|r| {
                    let output = match &r.kind {
                        RouteKind::Raw(route) => maybe_source_file(route).map(PathBuf::from),
                        RouteKind::Dir(DirRoute { dir, .. }) => Some(PathBuf::from(dir)),
                        RouteKind::Proxy(_) => None,
                    };

                    // exit early if there's no related path
                    let pb = output?;

                    let identity = server_config.identity.clone();

                    let spec = to_spec(&r.opts.watch);
                    let run = to_task_list(&spec);
                    let route_path = r.path.as_str().to_owned();

                    let route_watchable = RouteWatchable {
                        server_identity: identity,
                        route_path,
                        dir: pb,
                        spec,
                        task_list: run,
                    };

                    tracing::trace!(?route_watchable);

                    Some(route_watchable)
                })
        })
        .collect()
}

pub fn maybe_source_file(raw_route: &RawRoute) -> Option<&Path> {
    // todo(Shane): support file: in more places for directly serving
    match raw_route {
        RawRoute::Sse {
            sse: SseOpts { body, .. },
        } => body.split_once("file:").map(|(_, path)| Path::new(path)),
        _ => None,
    }
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
