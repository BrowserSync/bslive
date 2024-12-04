use std::env::current_dir;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RuntimeCtx {
    cwd: PathBuf,
}

impl Default for RuntimeCtx {
    fn default() -> Self {
        Self {
            cwd: current_dir().expect("failed to get current directory"),
        }
    }
}

impl RuntimeCtx {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self { cwd: path.into() }
    }
    pub fn cwd(&self) -> &PathBuf {
        &self.cwd
    }
}
