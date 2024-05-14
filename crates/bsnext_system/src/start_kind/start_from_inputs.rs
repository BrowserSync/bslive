use crate::startup::{StartupContext, SystemStart};

use bsnext_input::{Input, InputError};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct StartFromInputPaths {
    pub input_paths: Vec<String>,
}

impl SystemStart for StartFromInputPaths {
    fn input(&self, ctx: &StartupContext) -> Result<(Input, Option<PathBuf>), InputError> {
        let (input, path) = from_yml_paths(&ctx.cwd, &self.input_paths)?;
        Ok((input, Some(path)))
    }
}

fn from_yml_paths<T: AsRef<str>>(cwd: &Path, inputs: &[T]) -> Result<(Input, PathBuf), InputError> {
    let input_candidates = inputs
        .iter()
        .map(|path| cwd.join(path.as_ref()))
        .collect::<Vec<PathBuf>>();

    let lookups = ["input.yml", "input.yaml"]
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
        return Err(InputError::NotFound(
            missing.first().expect("guarded").to_path_buf(),
        ));
    }

    let first_user = exists.first();
    let first_auto = auto_candidates.first();

    let Some(input_path) = first_user.or(first_auto) else {
        return Err(InputError::MissingInputs);
    };

    tracing::info!(?input_path);

    let result = Input::from_input_path(input_path);
    match result {
        Ok(input) => Ok((input, input_path.to_path_buf())),
        Err(e) => Err(InputError::InvalidInput(e.to_string())),
    }
}
