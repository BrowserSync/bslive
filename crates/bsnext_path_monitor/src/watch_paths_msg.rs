use std::path::PathBuf;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct WatchPaths {
    pub paths: Vec<PathBuf>,
}
