use crate::{Input, InputError};
use std::env::current_dir;
use std::fmt::{Debug, Formatter};
use std::path::PathBuf;

pub struct Startup {
    pub tasks: Vec<StartupTask>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct StartupContext {
    pub cwd: PathBuf,
}

impl StartupContext {
    pub fn new(cwd: impl Into<PathBuf>) -> Self {
        Self { cwd: cwd.into() }
    }
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
    InputOnlyDeferred {
        input: Input,
        create: Lazy,
    },
    PathWithInvalidInput {
        path: PathBuf,
        input_error: InputError,
    },
    RunOnly {
        input: Input,
        named: Vec<String>,
        run_mode: RunMode,
        top_level_run_mode: TopLevelRunMode,
    },
}

pub struct Lazy {
    inner: Box<dyn Fn(Input) -> Result<Input, Box<InputError>>>,
}

impl Debug for Lazy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Lazy")
    }
}

impl Lazy {
    pub fn new(inner: Box<dyn Fn(Input) -> Result<Input, Box<InputError>>>) -> Self {
        Self { inner }
    }
    pub fn exec(self, input: Input) -> Result<Input, Box<InputError>> {
        (*self.inner)(input)
    }
}

#[derive(Debug, Clone)]
pub enum RunMode {
    Exec { preview: bool, summary: bool },
    Dry,
}

#[derive(Debug, Clone)]
pub enum TopLevelRunMode {
    Seq,
    All,
}

pub trait SystemStart {
    fn resolve_input(&self, ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>>;
}

impl Default for StartupContext {
    fn default() -> Self {
        Self::from_cwd(None)
    }
}

pub enum StartupTask {}
