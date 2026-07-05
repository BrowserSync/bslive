use bsnext_fs::{BufferedChangeEvent, Debounce, FsEvent, FsEventContext, PathDescriptionOwned};
use bsnext_input::route::WatchSpec;
pub mod path_and_filter;
pub mod path_monitor;
pub mod watch_paths_msg;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct PathMonitorEvent {
    pub watch_spec: WatchSpec,
    pub debounce: Debounce,
    pub group: Group,
}

#[derive(Debug, Clone)]
pub enum Group {
    Singular(FsEvent),
    BufferedChange(BufferedChangeEvent),
}

impl PathMonitorEvent {
    pub fn singular(evt: FsEvent, watch_spec: WatchSpec, debounce: Debounce) -> Self {
        PathMonitorEvent {
            debounce,
            group: Group::Singular(evt),
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
            group: Group::BufferedChange(BufferedChangeEvent {
                events,
                fs_event_ctx,
            }),
            watch_spec,
            debounce,
        }
    }
}
