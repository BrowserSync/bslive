use crate::buffered_debounce::BufferedStreamOpsExt;
use crate::inner_fs_event_handler::{InnerChangeEvent, MultipleInnerChangeEvent};
use crate::stream::StreamOpsExt;
use crate::{watcher, Debounce, FsEvent, FsEventContext};
use actix::{Actor, Addr, AsyncContext, Recipient, Running};
use actix_rt::Arbiter;
use futures_util::StreamExt;

use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::filter::Filter;
use tokio::sync::{broadcast, mpsc};
use tokio_stream::wrappers::ReceiverStream;

pub struct FsWatcher {
    pub watcher: Option<notify::FsEventWatcher>,
    raw_fs_stream: Arc<broadcast::Sender<InnerChangeEvent>>,
    pub receivers: Vec<Recipient<FsEvent>>,
    pub ctx: FsEventContext,
    pub debounce: Debounce,
    pub filters: Vec<Filter>,
    pub cwd: PathBuf,
}

impl FsWatcher {
    pub fn new(cwd: &Path, id: u64) -> Self {
        // todo: does this need to be a broadcast::channel? would an unbounded mpsc be better?
        let (raw_fs_sender, _) = broadcast::channel::<InnerChangeEvent>(1000);

        Self {
            watcher: None,
            raw_fs_stream: Arc::new(raw_fs_sender),
            receivers: vec![],
            ctx: FsEventContext::Other { id },
            cwd: cwd.to_path_buf(),
            filters: vec![],
            debounce: Default::default(),
        }
    }

    pub fn for_input(cwd: &Path, id: u64) -> Self {
        let mut s = Self::new(cwd, id);
        s.ctx = FsEventContext::InputFile { id };
        s
    }

    pub fn with_debounce(&mut self, d: Debounce) {
        self.debounce = d
    }

    pub fn with_filter(&mut self, f: Filter) {
        tracing::debug!("adding filter {:?}", f);
        self.filters.push(f)
    }

    fn setup_fs_streams(&mut self, a: Addr<FsWatcher>) {
        let raw_fs_events_sender = self.raw_fs_stream.clone();
        let (debounce_sender, debounce_receiver) = mpsc::channel::<InnerChangeEvent>(1);

        Arbiter::current().spawn({
            let debounce = self.debounce;
            async move {
                match debounce {
                    Debounce::Trailing { duration } => {
                        let stream = ReceiverStream::new(debounce_receiver).debounce(duration);
                        let mut debounced_stream = Box::pin(stream);
                        while let Some(v) = debounced_stream.next().await {
                            a.do_send(v);
                        }
                    }
                    Debounce::Buffered { duration } => {
                        let stream =
                            ReceiverStream::new(debounce_receiver).buffered_debounce(duration);
                        let mut debounced_stream = Box::pin(stream);
                        while let Some(v) = debounced_stream.next().await {
                            a.do_send(MultipleInnerChangeEvent { events: v })
                        }
                    }
                }
            }
        });

        // Listen for noisy FS events and pump them into the broadcast channel
        Arbiter::current().spawn({
            async move {
                let debounce_sender = debounce_sender.clone();
                let mut raw_fs_receiver = raw_fs_events_sender.subscribe();
                while let Ok(raw_event) = raw_fs_receiver.recv().await {
                    tracing::trace!(?raw_event);
                    match debounce_sender.send(raw_event).await {
                        Ok(_) => {}
                        Err(e) => tracing::error!(?e),
                    }
                }
            }
        });
    }
}

impl Actor for FsWatcher {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        tracing::trace!(actor.name = "FsWatcher", actor.lifecyle = "started");
        let self_address = ctx.address();
        let rx2 = self.raw_fs_stream.clone();
        match watcher::create_watcher(rx2, &self.cwd) {
            Ok(watcher) => self.watcher = Some(watcher),
            Err(e) => tracing::error!(?e, "could not create watcher"),
        }
        self.setup_fs_streams(self_address);
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        tracing::trace!(actor.name = "FsWatcher", actor.lifecyle = "stopping");
        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(actor.name = "FsWatcher", actor.lifecyle = "stopped");
    }
}
