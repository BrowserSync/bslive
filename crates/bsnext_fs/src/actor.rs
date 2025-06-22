use crate::filter::PathFilter;
use crate::inner_fs_event_handler::InnerChangeEvent;
use crate::{watcher, FsEvent, FsEventContext, FsEventKind, PathDescription, PathDescriptionOwned};
use actix::{Actor, Recipient, Running};
use actix_rt::Arbiter;
use std::fmt::{Display, Formatter};

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::filter::Filter;
use tokio::sync::broadcast;

#[derive(Debug)]
pub struct FsWatcher {
    pub watcher: Option<notify::RecommendedWatcher>,
    raw_fs_stream: Arc<broadcast::Sender<InnerChangeEvent>>,
    pub receiver: Recipient<FsEvent>,
    pub ctx: FsEventContext,
    pub filters: Vec<Filter>,
    pub ignore: Vec<Filter>,
    pub cwd: PathBuf,
}

impl Display for FsWatcher {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "FsWatcher {{ cwd: '{}' }}", self.cwd.display(),)
    }
}

impl FsWatcher {
    pub fn new(cwd: &Path, ctx: FsEventContext, receiver: Recipient<FsEvent>) -> Self {
        // todo: does this need to be a broadcast::channel? would an unbounded mpsc be better?
        let (raw_fs_sender, _) = broadcast::channel::<InnerChangeEvent>(1000);

        Self {
            watcher: None,
            raw_fs_stream: Arc::new(raw_fs_sender),
            receiver,
            ctx,
            cwd: cwd.to_path_buf(),
            filters: vec![],
            ignore: vec![],
        }
    }

    pub fn for_root(cwd: &Path, id: u64, receiver: Recipient<FsEvent>) -> Self {
        let ctx = FsEventContext { id, origin_id: id };
        Self::new(cwd, ctx, receiver)
    }

    pub fn with_filter(&mut self, f: Filter) {
        tracing::debug!("adding filter {:?}", f);
        self.filters.push(f)
    }
    pub fn with_ignore(&mut self, f: Filter) {
        tracing::debug!("adding ignore {:?}", f);
        self.ignore.push(f)
    }

    fn setup_fs_streams(&mut self) {
        let raw_fs_events_sender = self.raw_fs_stream.clone();
        let ctx = self.ctx;

        // Listen for noisy FS events sen
        Arbiter::current().spawn({
            let receiver = self.receiver.clone();
            let cwd = self.cwd.clone();
            let filters = self.filters.clone();
            let ignore = self.ignore.clone();
            async move {
                let mut raw_fs_receiver = raw_fs_events_sender.subscribe();
                while let Ok(raw_event) = raw_fs_receiver.recv().await {
                    tracing::trace!(?raw_event);
                    let relative = raw_event.absolute_path.strip_prefix(&cwd).ok();
                    let pd = PathDescription {
                        absolute: &raw_event.absolute_path,
                        relative,
                    };
                    let any_match = if filters.is_empty() {
                        true
                    } else {
                        filters.iter().any(|filter| filter.any(&pd))
                    };
                    let ignored = {
                        if ignore.is_empty() {
                            false
                        } else {
                            ignore.iter().any(|filter| filter.any(&pd))
                        }
                    };

                    tracing::trace!("any_matches: count: {}: {any_match}", filters.len());
                    tracing::trace!("ignored: count: {}: {ignored}", ignore.len());

                    if any_match && !ignored {
                        let as_fs_event = FsEvent {
                            kind: FsEventKind::Change(PathDescriptionOwned::from(&pd)),
                            fs_event_ctx: ctx,
                        };
                        receiver.do_send(as_fs_event);
                    } else {
                        tracing::trace!(
                            "not forwarding this event. filtered: {any_match}, ignored: {ignored}"
                        );
                    }
                }
            }
        });
    }
}

impl Actor for FsWatcher {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(actor.name = "FsWatcher", actor.lifecyle = "started");
        let rx2 = self.raw_fs_stream.clone();
        match watcher::create_watcher(rx2, &self.cwd) {
            Ok(watcher) => self.watcher = Some(watcher),
            Err(e) => tracing::error!(?e, "could not create watcher"),
        }
        self.setup_fs_streams();
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        tracing::trace!(actor.name = "FsWatcher", actor.lifecyle = "stopping");
        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(actor.name = "FsWatcher", actor.lifecyle = "stopped");
    }
}
