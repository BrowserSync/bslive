use bsnext_input::Input;
use bsnext_input::route::{DirRoute, PathPattern, RawRoute, RouteKind, SseOpts, WatchSpec};
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::watch_opts::WatchOpts;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct RouteWatchable {
    pub server_identity: ServerIdentity,
    pub route_path: String,
    pub dir: PathBuf,
    pub watch_spec: WatchSpec,
}

pub fn to_route_watchables(input: &Input) -> Vec<RouteWatchable> {
    input
        .servers
        .iter()
        .flat_map(|server_config| {
            server_config
                .routes
                .iter()
                .filter(|route| route.opts.watch.is_enabled())
                .filter_map(|route| {
                    let output = match &route.kind {
                        RouteKind::Raw(route) => maybe_source_file(route).map(PathBuf::from),
                        RouteKind::Dir(DirRoute { dir, .. }) => Some(PathBuf::from(dir)),
                        RouteKind::Proxy(_) => None,
                    };

                    // exit early if there's no related path
                    let pb = output?;

                    let identity = server_config.identity.clone();
                    let watch_spec = to_watch_spec(&route.opts.watch).with_globals(&input.config);
                    let route_path = route.path.as_str().to_owned();

                    let route_watchable = RouteWatchable {
                        server_identity: identity,
                        route_path,
                        dir: pb,
                        watch_spec,
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

pub fn to_watch_spec(wo: &WatchOpts) -> WatchSpec {
    match wo {
        WatchOpts::Bool(enabled) if !*enabled => unreachable!("should be handled..."),
        WatchOpts::Bool(enabled) if *enabled => WatchSpec::default(),
        WatchOpts::InlineGlob(glob) => WatchSpec {
            debounce: None,
            only: Some(PathPattern::Glob {
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
