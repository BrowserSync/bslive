use std::path::PathBuf;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct InputWatchable {
    pub path: PathBuf,
}
