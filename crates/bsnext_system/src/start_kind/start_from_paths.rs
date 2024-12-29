use crate::start_kind::fs_write_input;
use bsnext_fs_helpers::WriteMode;
use bsnext_input::path_def::PathDef;
use bsnext_input::route::{DirRoute, Opts, Route, RouteKind};
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
use bsnext_input::target::TargetKind;
use bsnext_input::{Input, InputError, PathDefinition, PathDefs, PathError};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct StartFromTrailingArgs {
    pub paths: Vec<String>,
    pub write_input: bool,
    pub port: Option<u16>,
    pub force: bool,
    pub route_opts: Opts,
}

impl SystemStart for StartFromTrailingArgs {
    fn input(&self, ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        let identity =
            ServerIdentity::from_port_or_named(self.port).map_err(|e| Box::new(e.into()))?;
        let input = from_dir_paths(&ctx.cwd, &self.paths, &self.route_opts, identity)
            .map_err(|e| Box::new(e.into()))?;
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
            tracing::info!("SystemStartArgs::InputOnly");
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
    Ok(Input {
        servers: vec![server],
    })
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
        };
        let ctx = StartupContext {
            cwd: tmp_dir.path().to_path_buf(),
        };
        let i = v.input(&ctx);
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
