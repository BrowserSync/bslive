use bsnext_fs::{BufferedChangeset, Debounce, FsEvent, FsEventContext, PathDescriptionOwned};
use bsnext_input::route::WatchSpec;
pub mod path_and_filter;
pub mod path_monitor;
pub mod watch_paths_msg;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct PathMonitorChangeset {
    pub watch_spec: WatchSpec,
    pub debounce: Debounce,
    pub changeset: Changeset,
}

#[derive(Debug, Clone)]
pub enum Changeset {
    Singular(FsEvent),
    BufferedChange(BufferedChangeset),
}

impl PathMonitorChangeset {
    pub fn singular(evt: FsEvent, watch_spec: WatchSpec, debounce: Debounce) -> Self {
        PathMonitorChangeset {
            debounce,
            changeset: Changeset::Singular(evt),
            watch_spec,
        }
    }
    pub fn buffered_change(
        events: Vec<PathDescriptionOwned>,
        fs_event_ctx: FsEventContext,
        watch_spec: WatchSpec,
        debounce: Debounce,
    ) -> Self {
        Self {
            changeset: Changeset::BufferedChange(BufferedChangeset {
                changes: events,
                fs_event_ctx,
            }),
            watch_spec,
            debounce,
        }
    }
}
