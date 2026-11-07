use std::path::PathBuf;

#[derive(Debug, Clone)]
pub enum InputSrc {
    File(FileInputSrc),
}

#[derive(Debug, Clone)]
pub struct FileInputSrc {
    pub absolute: PathBuf,
}

impl FileInputSrc {
    pub fn new(abs: impl Into<PathBuf>) -> Self {
        Self {
            absolute: abs.into(),
        }
    }
}
