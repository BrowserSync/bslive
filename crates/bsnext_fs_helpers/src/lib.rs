#![allow(unused)]
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum FsWriteError {
    #[error("couldn't write file to {path}")]
    FailedWrite { path: PathBuf },
    #[error("couldn't create dir {0}")]
    FailedDir(#[from] std::io::Error),
    #[error("couldn't read the status of {path}")]
    CannotQueryStatus { path: PathBuf },
    #[error("file already exists, override with --force (dangerous) {path}")]
    Exists { path: PathBuf },
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum DirError {
    #[error("could not create that dir: {path}")]
    CannotCreate { path: PathBuf },
    #[error("could not change the process CWD to: {path}")]
    CannotMove { path: PathBuf },
    #[error("could not query the status")]
    CannotQueryStatus { path: PathBuf },
    #[error("directory already exists, override with --force (dangerous) {path}")]
    Exists { path: PathBuf },
}

#[derive(Default, Debug, PartialEq)]
pub enum WriteMode {
    #[default]
    Safe,
    Override,
}

pub fn fs_write_str(
    cwd: &Path,
    path: &Path,
    string: &str,
    write_mode: &WriteMode,
) -> Result<PathBuf, FsWriteError> {
    let next_path = cwd.join(path);
    tracing::info!(
        "✏️ writing {} bytes to {}",
        string.len(),
        next_path.display()
    );

    let exists = fs::exists(&next_path).map_err(|_e| FsWriteError::CannotQueryStatus {
        path: next_path.clone(),
    })?;

    if exists && *write_mode == WriteMode::Safe {
        return Err(FsWriteError::Exists { path: next_path });
    }

    fs::write(&next_path, string)
        .map(|()| next_path.clone())
        .map_err(|_e| FsWriteError::FailedWrite { path: next_path })
}

/// Create a directory and move the current process's CWD there
pub fn create_dir_and_cd(dir: &Path, write_mode: &WriteMode) -> Result<PathBuf, DirError> {
    let exists = fs::exists(dir).map_err(|_e| DirError::CannotQueryStatus {
        path: dir.to_path_buf(),
    })?;

    if exists && *write_mode == WriteMode::Safe {
        return Err(DirError::Exists {
            path: dir.to_path_buf(),
        });
    }

    fs::create_dir_all(dir)
        .map_err(|_e| DirError::CannotCreate {
            path: dir.to_path_buf(),
        })
        .and_then(|_pb| {
            std::env::set_current_dir(dir).map_err(|_e| DirError::CannotMove {
                path: dir.to_path_buf(),
            })
        })
        .map(|_| dir.to_path_buf())
}
