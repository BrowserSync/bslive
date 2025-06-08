pub mod actor;
mod buffered_debounce;
pub mod filter;
pub mod inner_fs_event_handler;
pub mod remove_path_handler;
pub mod stop_handler;
mod stream;
#[cfg(test)]
mod test;
pub mod watch_path_handler;
mod watcher;

use std::path::{Path, PathBuf};
use std::time::Duration;
// use tokio_stream::StreamExt;

#[derive(Debug, Copy, Clone)]
pub enum Debounce {
    Trailing { duration: Duration },
    Buffered { duration: Duration },
}

impl Default for Debounce {
    fn default() -> Self {
        Self::Trailing {
            duration: Duration::from_millis(300),
        }
    }
}

impl Debounce {
    pub fn trailing_ms(ms: u64) -> Self {
        Self::Trailing {
            duration: Duration::from_millis(ms),
        }
    }
    pub fn buffered_ms(ms: u64) -> Self {
        Self::Buffered {
            duration: Duration::from_millis(ms),
        }
    }
}

impl Debounce {
    pub fn duration(&self) -> &Duration {
        match self {
            Debounce::Trailing { duration } => duration,
            Debounce::Buffered { duration } => duration,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct FsEventContext {
    pub id: u64,
    pub origin_id: u64,
}

impl FsEventContext {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn is_root(&self) -> bool {
        self.id == 0 && self.origin_id == 0
    }
}

impl Default for FsEventContext {
    fn default() -> Self {
        Self {
            id: 1,
            origin_id: 1,
        }
    }
}

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct FsEvent {
    pub kind: FsEventKind,
    pub fs_event_ctx: FsEventContext,
}

impl FsEvent {
    pub fn changed<A: AsRef<Path>>(abs: A, path: A, ctx_id: u64) -> Self {
        Self {
            kind: FsEventKind::Change(ChangeEvent {
                absolute_path: PathBuf::from(abs.as_ref()),
                path: PathBuf::from(path.as_ref()),
            }),
            fs_event_ctx: FsEventContext {
                id: ctx_id,
                origin_id: ctx_id,
            },
        }
    }
}

#[derive(Debug, Clone)]
pub enum FsEventKind {
    Change(ChangeEvent),
    ChangeBuffered(BufferedChangeEvent),
    PathAdded(PathAddedEvent),
    PathRemoved(PathEvent),
    PathNotFoundError(PathEvent),
}

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct ChangeEvent {
    pub absolute_path: PathBuf,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PathDescription<'a> {
    pub absolute: &'a Path,
    pub relative: Option<&'a Path>,
}

#[derive(Debug, Clone)]
pub struct PathDescriptionOwned {
    pub absolute: PathBuf,
    pub relative: Option<PathBuf>,
}

impl<'a> From<&'a PathDescription<'_>> for PathDescriptionOwned {
    fn from(value: &'a PathDescription<'_>) -> Self {
        Self {
            relative: value.relative.map(ToOwned::to_owned),
            absolute: value.absolute.to_owned(),
        }
    }
}

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct BufferedChangeEvent {
    pub events: Vec<PathDescriptionOwned>,
}

impl BufferedChangeEvent {
    pub fn dropping_absolute(self, path: &Path) -> Self {
        if self.events.iter().any(|x| x.absolute == path) {
            Self {
                events: self
                    .events
                    .iter()
                    .filter(|x| x.absolute != path)
                    .map(ToOwned::to_owned)
                    .collect(),
            }
        } else {
            Self {
                events: self.events,
            }
        }
    }
}

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct PathAddedEvent {
    pub path: PathBuf,
    pub debounce: Debounce,
}

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct PathEvent {
    pub path: PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum FsWatchError {
    #[error("Watcher error, original error: {0}")]
    Watcher(#[from] notify::Error),
}
