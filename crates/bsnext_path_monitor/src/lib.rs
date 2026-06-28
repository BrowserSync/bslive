use crate::path_monitor::PathMonitor;
use crate::watchables::path_watchable::PathWatchable;
use actix::Addr;
use bsnext_fs::{BufferedChangeEvent, Debounce, FsEvent, FsEventContext, PathDescriptionOwned};
use bsnext_input::route::WatchSpec;
use std::collections::HashMap;

pub mod monitor_path_watchables;
pub mod path_monitor;
pub mod watchables;

#[derive(Debug, Default)]
pub struct Monitor {
    path_monitors: HashMap<PathWatchable, Addr<PathMonitor>>,
}

impl Monitor {
    pub fn new() -> Self {
        Self {
            path_monitors: Default::default(),
        }
    }
}

impl actix::Actor for Monitor {
    type Context = actix::Context<Self>;
}

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct FsGroup {
    pub watch_spec: WatchSpec,
    pub debounce: Debounce,
    pub group: Group,
}

#[derive(Debug, Clone)]
pub enum Group {
    Singular(FsEvent),
    BufferedChange(BufferedChangeEvent),
}

impl FsGroup {
    pub fn singular(evt: FsEvent, watch_spec: WatchSpec, debounce: Debounce) -> Self {
        FsGroup {
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
