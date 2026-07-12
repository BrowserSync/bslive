use crate::start::SystemStart;
use crate::watch::watch_sub_opts::WatchSubOpts;
use bsnext_input::path_def::PathDef;
use bsnext_input::route::{DirRoute, MultiWatch, Opts, Route, RouteKind};
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use bsnext_input::startup::{StartupContext, SystemStartArgs};
use bsnext_input::{
    InferWatchers, Input, InputError, PathDefinition, PathDefs, PathError, WatchGlobalConfig,
};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct StartFromPaths {
    pub paths: Vec<String>,
    pub port: Option<u16>,
    pub route_opts: Opts,
    pub watch_sub_opts: WatchSubOpts,
    pub no_watch: bool,
}

impl SystemStart for StartFromPaths {
    fn resolve_input(&self, _ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        // let span = tracing::debug_span!("StartFromPaths::resolve_input");
        // let _g = span.entered();
        //
        // let port = self.port;
        // let paths = self.paths.clone();
        // let route_opts = self.route_opts.clone();
        // let cwd = ctx.cwd.clone();
        //
        // // defer the lookup of directories or ports because a 'before' task might have sideeffects
        // let lazy = move |mut input: Input| -> Result<Input, Box<InputError>> {
        //     let identity =
        //         ServerIdentity::from_port_or_named(port).map_err(|e| Box::new(e.into()))?;
        //
        //     let server_config = server_config_from_paths(&cwd, &paths, &route_opts, identity)
        //         .map_err(|e| Box::new(e.into()))?;
        //
        //     tracing::debug!("Adding a `server_config`");
        //     input.servers.push(server_config);
        //
        //     Ok(input)
        // };
        //
        // let input = if self.no_watch {
        //     let mut input = Input::default();
        //     input.config.watchers = WatchGlobalConfig::Disabled;
        //     input
        // } else {
        //     let explicit_watch_count =
        //         self.watch_sub_opts.paths.len() + self.watch_sub_opts.before.len();
        //     if explicit_watch_count > 0 {
        //         with_explicit_paths(&self.watch_sub_opts)
        //     } else {
        //         with_inferred_watchers(&self.watch_sub_opts)
        //     }
        // };
        //
        // Ok(SystemStartArgs::InputOnlyDeferred {
        //     input,
        //     create: Lazy::new(Box::new(lazy)),
        // })
        todo!("never...")
    }

    // async fn start(&self) -> Result<DidStart, StartupError> {
    // let system = BsSystem::new(events_sender.clone(), cwd, tx);
    // let sys_addr = system.start();
    // let mut input = if self.no_watch {
    //     let mut input = Input::default();
    //     input.config.watchers = WatchGlobalConfig::Disabled;
    //     input
    // } else {
    //     let explicit_watch_count =
    //         self.watch_sub_opts.paths.len() + self.watch_sub_opts.before.len();
    //     if explicit_watch_count > 0 {
    //         with_explicit_paths(&self.watch_sub_opts)
    //     } else {
    //         with_inferred_watchers(&self.watch_sub_opts)
    //     }
    // };
    // let res = setup_jobs_only(addr.clone(), input.clone()).await?;
    // let port = self.port;
    // let paths = self.paths.clone();
    // let route_opts = self.route_opts.clone();
    // let cwd = ctx.cwd.clone();
    //
    // let identity = ServerIdentity::from_port_or_named(port)
    //     .map_err(|e| StartupError::InputError(InputError::PortError(e)))?;
    // let server_config = from_dir_paths(&cwd, &paths, &route_opts, identity)
    //     .map_err(|e| StartupError::InputError(InputError::PathError(e)))?;
    //
    // tracing::debug!("Adding a `server_config` on the fly");
    // input.servers.push(server_config);
    //
    // let SetupServersOk {
    //     servers,
    //     child_results,
    // } = setup_servers_only(server_addr.clone(), input.clone()).await?;
    //
    // // let notif = ServerChangesetDTO::from_changes(&servers, &child_results);
    // let errored = ChildResult::first_server_error(&child_results);
    //
    // // dbg!(&notif);
    //
    // if let Some(server_error) = errored {
    //     tracing::debug!("errored: {:?}", errored);
    //     return Err(StartupError::ServerError((*server_error).to_owned()));
    // }
    // ctx.notify(MonitorAny::new(res.input.clone()));
    // Ok(DidStart::Started)
    // }
}

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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test() -> anyhow::Result<()> {
        use tempfile::tempdir;
        let tmp_dir = tempdir()?;
        let v = StartFromPaths {
            paths: vec![".".into()],
            port: Some(3000),
            route_opts: Default::default(),
            watch_sub_opts: Default::default(),
            no_watch: false,
        };
        let ctx = StartupContext {
            cwd: tmp_dir.path().to_path_buf(),
        };
        let i = v.resolve_input(&ctx);
        let start_args = i.unwrap();
        if let SystemStartArgs::InputOnlyDeferred { input, create } = start_args {
            let input = create.exec(input)?;
            tmp_dir.close()?;
            insta::assert_debug_snapshot!(input);
            insta::assert_yaml_snapshot!(input);
        } else {
            unreachable!("cannot get here?")
        }
        Ok(())
    }
}
