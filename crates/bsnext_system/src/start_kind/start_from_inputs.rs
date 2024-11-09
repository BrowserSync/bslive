use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};

use crate::input_fs::from_input_path;
use bsnext_input::{Input, InputError};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct StartFromInputPaths {
    pub input_paths: Vec<String>,
}

impl SystemStart for StartFromInputPaths {
    fn input(&self, ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        from_yml_paths(&ctx.cwd, &self.input_paths)
    }
}

#[derive(Debug)]
pub struct StartFromInput {
    pub input: Input,
}

impl SystemStart for StartFromInput {
    fn input(&self, _ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        Ok(SystemStartArgs::InputOnly {
            input: self.input.clone(),
        })
    }
}

fn from_yml_paths<T: AsRef<str>>(
    cwd: &Path,
    inputs: &[T],
) -> Result<SystemStartArgs, Box<InputError>> {
    let input_candidates = inputs
        .iter()
        .map(|path| cwd.join(path.as_ref()))
        .collect::<Vec<PathBuf>>();

    let lookups = ["bslive.yml", "bslive.yaml"]
        .iter()
        .map(|path| cwd.join(path))
        .collect::<Vec<PathBuf>>();

    let auto_candidates = lookups
        .iter()
        .filter(|path| Path::exists(path))
        .collect::<Vec<&PathBuf>>();

    let exists = input_candidates
        .iter()
        .filter(|path| Path::exists(path))
        .collect::<Vec<&PathBuf>>();

    let missing = input_candidates
        .iter()
        .filter(|path| !Path::exists(path))
        .collect::<Vec<&PathBuf>>();

    if !missing.is_empty() {
        for path in &missing {
            tracing::error!(?path, "input file not found");
        }
        return Err(Box::new(InputError::NotFound(
            missing.first().expect("guarded").to_path_buf(),
        )));
    }

    let first_user = exists.first();
    let first_auto = auto_candidates.first();

    let Some(input_path) = first_user.or(first_auto) else {
        return Err(Box::new(InputError::MissingInputs));
    };

    tracing::info!(?input_path);

    let result = from_input_path(input_path);
    match result {
        Ok(input) => Ok(SystemStartArgs::PathWithInput {
            path: input_path.to_path_buf(),
            input,
        }),
        Err(e) => match *e {
            InputError::YamlError(yaml_error) => Ok(SystemStartArgs::PathWithInvalidInput {
                path: input_path.to_path_buf(),
                input_error: InputError::YamlError(yaml_error),
            }),
            _ => {
                tracing::error!("cannot continue");
                Err(e)
            }
        },
    }
}
