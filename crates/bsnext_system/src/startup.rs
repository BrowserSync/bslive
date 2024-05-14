use bsnext_input::{Input, InputError};
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

pub trait SystemStart {
    fn input(&self, ctx: &StartupContext) -> Result<(Input, Option<PathBuf>), InputError>;
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
    #[error("An input error prevented startup")]
    InputError(#[from] InputError),
}

pub enum StartupTask {}
