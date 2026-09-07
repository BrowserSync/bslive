use crate::watch::watch_sub_opts::WatchSubOpts;
use bsnext_input::path_def::PathDef;
use bsnext_input::route::{DirRoute, MultiWatch, Opts, Route, RouteKind};
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use bsnext_input::{InferWatchers, Input, PathDefinition, PathDefs, PathError, WatchGlobalConfig};
use std::path::{Path, PathBuf};

/// when we have given explicit paths on the command line, we are opting out of inferred watchers,
/// for example, the following command would normally create a watcher for the path 'public'
///   `bslive public`
/// however, if we want to watch something else, we need a way to opt-out of the inferred ones, like:
///   `bslive public --watch.paths src`
///                      ^ in this case, we only want to watch 'src' and prevent inferred things later.
pub fn with_explicit_paths(input: &mut Input, opts: &WatchSubOpts) {
    let span = tracing::debug_span!(parent: None, "StartFromPaths watch_overrides");
    let _g = span.entered();
    tracing::debug!("{} paths to watch", opts.paths.len());
    tracing::debug!("{} sh_commands to run", opts.run.len());
    let multi = MultiWatch::from(opts.clone());
    input.watchers = vec![multi];
    input.config.watchers = WatchGlobalConfig::Enabled {
        infer: InferWatchers::None,
    };
}

/// in this second scenario, we're probably starting a server without explicit watch paths given as CLI args.
///   eg: `bslive .`
/// this means we'll watch the directory, but it might accidentally end up watching a noisy build folder or similar
/// in that case, we still want to forward --ignore and --only args, but nothing else to allow the user to opt-out
///   eg: `bslive . --only dist/*.html`
pub fn with_inferred_watchers(input: &mut Input, opts: &WatchSubOpts) {
    let span = tracing::debug_span!(parent: None, "StartFromPaths global_overrides");
    let _g = span.entered();
    tracing::debug!("{} ignore add", opts.ignore.len());
    let multi = MultiWatch::from(opts.clone());
    if let Some(spec_from_cli) = multi.spec {
        input.config.global_fs_ignore = spec_from_cli.ignore;
        input.config.global_fs_only = spec_from_cli.only;
        input.config.global_fs_debounce = spec_from_cli.debounce;
    }
}

pub fn server_config_from_paths<T: AsRef<str>>(
    cwd: &Path,
    paths: &[T],
    route_opts: &Opts,
    identity: ServerIdentity,
) -> Result<ServerConfig, PathError> {
    let path_defs = paths
        .iter()
        .map(|p| {
            let pb = PathBuf::from(p.as_ref());
            if pb.is_absolute() {
                PathDefinition {
                    input: p.as_ref().to_string(),
                    cwd: cwd.to_path_buf(),
                    absolute: pb,
                }
            } else {
                PathDefinition {
                    input: p.as_ref().to_string(),
                    cwd: cwd.to_path_buf(),
                    absolute: cwd.join(pb),
                }
            }
        })
        .map(|path_def| {
            let exists = path_def.absolute.exists();
            (path_def, exists)
        })
        .collect::<Vec<(PathDefinition, bool)>>();

    let invalid = path_defs
        .into_iter()
        .filter_map(|(pb, exists)| if exists { None } else { Some(pb) })
        .collect::<Vec<_>>();

    if !invalid.is_empty() {
        tracing::info!("bailing because a path wasn't found {:?}", invalid);
        return Err(PathError::MissingPaths {
            paths: PathDefs(invalid),
        });
    }

    let server = ServerConfig {
        identity,
        routes: paths
            .iter()
            .map(|p| -> Route {
                let str = p.as_ref();
                Route {
                    path: PathDef::root(),
                    kind: RouteKind::Dir(DirRoute {
                        dir: str.into(),
                        base: None,
                    }),
                    opts: route_opts.clone(),
                    ..Default::default()
                }
            })
            .collect(),
        ..Default::default()
    };
    Ok(server)
}
