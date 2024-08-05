use crate::{Input, InputError};
use std::env::current_dir;
use std::path::PathBuf;

pub type StartupResult = Result<DidStart, StartupError>;

pub struct Startup {
    pub tasks: Vec<StartupTask>,
}

#[derive(Debug)]
pub struct StartupContext {
    pub cwd: PathBuf,
}

impl StartupContext {
    pub fn from_cwd(cwd: Option<&PathBuf>) -> Self {
        StartupContext {
            cwd: cwd.map(ToOwned::to_owned).unwrap_or_else(|| {
                PathBuf::from(
                    current_dir()
                        .expect("if current_dir fails, nothing can work")
                        .to_string_lossy()
                        .to_string(),
                )
            }),
        }
    }
}

#[derive(Debug)]
pub enum SystemStartArgs {
    PathWithInput {
        path: PathBuf,
        input: Input,
    },
    InputOnly {
        input: Input,
    },
    PathWithInvalidInput {
        path: PathBuf,
        input_error: InputError,
    },
}

pub trait SystemStart {
    fn input(&self, ctx: &StartupContext) -> Result<SystemStartArgs, InputError>;
}

impl Default for StartupContext {
    fn default() -> Self {
        Self::from_cwd(None)
    }
}

#[derive(Debug)]
pub enum DidStart {
    Started,
}

#[derive(Debug, thiserror::Error)]
pub enum StartupError {
    #[error("{0}")]
    InputError(#[from] InputError),
}

pub enum StartupTask {}
