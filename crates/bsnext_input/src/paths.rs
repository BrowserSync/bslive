use crate::route::{DirRoute, Route, RouteKind};
use crate::server_config::{Identity, ServerConfig};
use crate::{Input, PathDefinition, PathDefs, PathError};
use std::path::{Path, PathBuf};

pub fn from_paths<T: AsRef<str>>(
    cwd: &Path,
    paths: &[T],
    identity: Identity,
) -> Result<Input, PathError> {
    let path_defs = paths
        .iter()
        .map(|p| {
            let pb = PathBuf::from(p.as_ref());
            if pb.is_absolute() {
                return PathDefinition {
                    input: p.as_ref().to_string(),
                    cwd: cwd.to_path_buf(),
                    absolute: pb,
                };
            } else {
                return PathDefinition {
                    input: p.as_ref().to_string(),
                    cwd: cwd.to_path_buf(),
                    absolute: cwd.join(pb),
                };
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
                    path: "/".to_string(),
                    kind: RouteKind::Dir(DirRoute { dir: str.into() }),
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
