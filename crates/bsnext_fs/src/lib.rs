pub mod actor;
pub mod buffered_debounce;
pub mod filter;
pub mod inner_fs_event_handler;
pub mod stop_handler;
pub mod stream;
pub mod watch_path_handler;
mod watcher;

use std::path::{Path, PathBuf};
use std::time::Duration;
// use tokio_stream::StreamExt;

#[derive(Debug, Copy, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
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

#[derive(Debug, Clone, Copy, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub struct FsEventContext {
    pub id: u64,
    pub origin_id: u64,
}

impl FsEventContext {
    pub fn new(id: u64, origin_id: u64) -> Self {
        Self { id, origin_id }
    }
    pub fn for_root() -> Self {
        Self {
            id: 0,
            origin_id: 0,
        }
    }
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

#[derive(actix::Message, Debug, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
#[rtype(result = "()")]
pub struct FsEvent {
    pub kind: FsEventKind,
    pub fs_event_ctx: FsEventContext,
}

impl FsEvent {
    pub fn changed<A: AsRef<Path>>(absolute: A, relative: A, ctx_id: u64) -> Self {
        Self {
            kind: FsEventKind::Change(PathDescriptionOwned {
                absolute: PathBuf::from(absolute.as_ref()),
                relative: Some(PathBuf::from(relative.as_ref())),
            }),
            fs_event_ctx: FsEventContext {
                id: ctx_id,
                origin_id: ctx_id,
            },
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
pub enum FsEventKind {
    Change(PathDescriptionOwned),
    PathAdded(PathAddedEvent),
    PathRemoved(PathEvent),
    PathNotFoundError(PathEvent),
}

#[derive(Debug, Clone)]
pub struct PathDescription<'a> {
    pub absolute: &'a Path,
    pub relative: Option<&'a Path>,
}

pub struct Abs<'a>(pub &'a str);
pub struct Cwd<'a>(pub &'a str);

impl<'a> PathDescription<'a> {
    pub fn new_abs(abs: &'a str) -> Self {
        Self {
            relative: None,
            absolute: Path::new(abs),
        }
    }
    pub fn new_rel(abs: &'a str, rel: &'a str) -> Self {
        Self {
            relative: Some(Path::new(rel)),
            absolute: Path::new(abs),
        }
    }
    pub fn from_cwd(abs: &'a Abs, cwd: &'a Cwd) -> Self {
        let abs = Path::new(abs.0);
        match abs.strip_prefix(Path::new(cwd.0)) {
            Ok(stripped) => Self {
                absolute: abs,
                relative: Some(stripped),
            },
            Err(e) => {
                tracing::debug!(
                    "could not strip prefix when using PathDescription.from_cwd {:?}",
                    e
                );
                Self {
                    absolute: abs,
                    relative: None,
                }
            }
        }
    }
}

#[derive(Debug, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
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
pub enum FsEventGrouping {
    Singular(FsEvent),
    BufferedChange(BufferedChangeEvent),
}

impl FsEventGrouping {
    pub fn buffered_change(
        events: Vec<PathDescriptionOwned>,
        fs_event_context: FsEventContext,
    ) -> Self {
        Self::BufferedChange(BufferedChangeEvent {
            events,
            fs_ctx: fs_event_context,
        })
    }
}

#[derive(Debug, Clone)]
pub struct BufferedChangeEvent {
    pub events: Vec<PathDescriptionOwned>,
    pub fs_ctx: FsEventContext,
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
                fs_ctx: self.fs_ctx,
            }
        } else {
            Self {
                events: self.events,
                fs_ctx: self.fs_ctx,
            }
        }
    }
}

#[derive(actix::Message, Debug, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
#[rtype(result = "()")]
pub struct PathAddedEvent {
    pub path: PathBuf,
}

#[derive(actix::Message, Debug, Clone, Hash, PartialEq, PartialOrd, Eq, Ord)]
#[rtype(result = "()")]
pub struct PathEvent {
    pub path: PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum FsWatchError {
    #[error("Watcher error, original error: {0}")]
    Watcher(#[from] notify::Error),
}
