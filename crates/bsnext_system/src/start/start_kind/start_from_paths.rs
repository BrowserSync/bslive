use crate::start::start_kind::fs_write_input;
use crate::watch::watch_sub_opts::WatchSubOpts;
use bsnext_fs_helpers::WriteMode;
use bsnext_input::path_def::PathDef;
use bsnext_input::route::{DirRoute, MultiWatch, Opts, Route, RouteKind};
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
use bsnext_input::target::TargetKind;
use bsnext_input::{InferWatchers, Input, InputError, PathDefinition, PathDefs, PathError};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct StartFromTrailingArgs {
    pub paths: Vec<String>,
    pub write_input: bool,
    pub port: Option<u16>,
    pub force: bool,
    pub route_opts: Opts,
    pub watch_sub_opts: WatchSubOpts,
}

impl SystemStart for StartFromTrailingArgs {
    fn resolve_input(&self, ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        let span = tracing::debug_span!("StartFromTrailingArgs::resolve_input");
        let _g = span.entered();

        let identity =
            ServerIdentity::from_port_or_named(self.port).map_err(|e| Box::new(e.into()))?;

        let mut input = from_dir_paths(&ctx.cwd, &self.paths, &self.route_opts, identity)
            .map_err(|e| Box::new(e.into()))?;

        'watch_overrides: {
            // if there is some explicit --watch.path given, we want to ONLY support that
            if self.watch_sub_opts.paths.is_empty() {
                break 'watch_overrides;
            }

            let span = tracing::debug_span!(parent: None, "StartFromTrailingArgs watch_sub_opts");
            let _g = span.entered();
            tracing::debug!("{} paths to watch", self.watch_sub_opts.paths.len());
            tracing::debug!("{} sh_commands to run", self.watch_sub_opts.run.len());
            let multi = MultiWatch::from(self.watch_sub_opts.clone());
            input.watchers = vec![multi];
            input.config.infer_watchers = InferWatchers::None;
        }

        let write_mode = if self.force {
            WriteMode::Override
        } else {
            WriteMode::Safe
        };
        if self.write_input {
            let path = fs_write_input(&ctx.cwd, &input, TargetKind::Yaml, &write_mode)
                .map_err(|e| Box::new(e.into()))?;
            Ok(SystemStartArgs::PathWithInput { input, path })
        } else {
            Ok(SystemStartArgs::InputOnly { input })
        }
    }
}

fn from_dir_paths<T: AsRef<str>>(
    cwd: &Path,
    paths: &[T],
    route_opts: &Opts,
    identity: ServerIdentity,
) -> Result<Input, PathError> {
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
    Ok(Input::from_server(server))
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test() -> anyhow::Result<()> {
        use tempfile::tempdir;
        let tmp_dir = tempdir()?;
        let v = StartFromTrailingArgs {
            paths: vec![".".into()],
            write_input: false,
            port: Some(3000),
            force: false,
            route_opts: Default::default(),
            watch_sub_opts: Default::default(),
        };
        let ctx = StartupContext {
            cwd: tmp_dir.path().to_path_buf(),
        };
        let i = v.resolve_input(&ctx);
        tmp_dir.close()?;
        let start_args = i.unwrap();
        if let SystemStartArgs::InputOnly { input } = start_args {
            insta::assert_debug_snapshot!(input);
            insta::assert_yaml_snapshot!(input);
        } else {
            unreachable!("cannot get here?")
        }
        Ok(())
    }
}
