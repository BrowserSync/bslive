use crate::path_watchable::PathWatchable;
use actix::{Addr, AsyncContext, Handler, Recipient, Running};
use actix_rt::Arbiter;
use bsnext_fs::actor::FsWatcher;
use bsnext_fs::buffered_debounce::BufferedStreamOpsExt;
use bsnext_fs::filter::Filter;
use bsnext_fs::inner_fs_event_handler::{MultipleInnerChangeEvent, SingleInnerChangeEvent};
use bsnext_fs::stop_handler::StopWatcher;
use bsnext_fs::stream::StreamOpsExt;
use bsnext_fs::watch_path_handler::RequestWatchPath;
use bsnext_fs::{
    Debounce, FsEvent, FsEventContext, FsEventGrouping, FsEventKind, PathAddedEvent,
    PathDescriptionOwned, PathEvent,
};
use bsnext_input::route::{FilterKind, Spec};
use futures_util::StreamExt;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, debug_span};

#[derive(Debug)]
pub struct PathMonitor {
    pub(crate) cwd: PathBuf,
    pub(crate) addrs: Vec<Addr<FsWatcher>>,
    pub(crate) sys: Recipient<FsEventGrouping>,
    pub(crate) fs_ctx: FsEventContext,
    pub(crate) path_watchable: PathWatchable,
    pub(crate) debounce: Debounce,
    inner_sender: tokio::sync::mpsc::Sender<FsEvent>,
    inner_receiver: Option<tokio::sync::mpsc::Receiver<FsEvent>>,
}

#[derive(Debug)]
pub struct PathMonitorMeta {
    #[allow(dead_code)]
    pub cwd: PathBuf,
    pub fs_ctx: FsEventContext,
    #[allow(dead_code)]
    pub path_watchable: PathWatchable,
    pub debounce: Debounce,
}

impl From<&PathMonitor> for PathMonitorMeta {
    fn from(value: &PathMonitor) -> Self {
        Self {
            path_watchable: value.path_watchable.clone(),
            cwd: value.cwd.clone(),
            fs_ctx: value.fs_ctx,
            debounce: value.debounce,
        }
    }
}

impl actix::Actor for PathMonitor {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        for single_path in &self.path_watchable.watch_paths() {
            let watcher = to_watcher(
                &self.cwd,
                self.path_watchable.spec_opts(),
                self.fs_ctx,
                ctx.address().recipient(),
            );
            let watcher_addr = watcher.start();
            let watcher_addr_clone = watcher_addr.clone();
            self.addrs.push(watcher_addr);
            watcher_addr_clone.do_send(RequestWatchPath {
                path: single_path.to_path_buf(),
            });
        }
        Arbiter::current().spawn({
            let debounce = self.debounce;
            let a = ctx.address();
            let Some(receiver) = self.inner_receiver.take() else {
                panic!("impossible?")
            };
            async move {
                match debounce {
                    Debounce::Trailing { duration } => {
                        let stream = ReceiverStream::new(receiver).debounce(duration);
                        let mut debounced_stream = Box::pin(stream);
                        while let Some(event) = debounced_stream.next().await {
                            a.do_send(SingleInnerChangeEvent { event });
                        }
                    }
                    Debounce::Buffered { duration } => {
                        let stream = ReceiverStream::new(receiver).buffered_debounce(duration);
                        let mut debounced_stream = Box::pin(stream);
                        while let Some(events) = debounced_stream.next().await {
                            a.do_send(MultipleInnerChangeEvent { events });
                        }
                    }
                };
            }
        });
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        dbg!(&"stopped");
    }
}

fn convert(fk: &FilterKind) -> Vec<Filter> {
    match fk {
        FilterKind::StringDefault(string_default) => {
            if string_default.contains("*") {
                vec![Filter::Glob {
                    glob: string_default.to_string(),
                }]
            } else {
                vec![Filter::Any {
                    any: string_default.to_string(),
                }]
            }
        }
        FilterKind::Extension { ext } => vec![Filter::Extension {
            ext: ext.to_string(),
        }],
        FilterKind::Glob { glob } => vec![Filter::Glob {
            glob: glob.to_string(),
        }],
        FilterKind::List(items) => items.iter().flat_map(convert).collect::<Vec<_>>(),
        FilterKind::Any { any } => vec![Filter::Any {
            any: any.to_string(),
        }],
    }
}

impl Handler<MultipleInnerChangeEvent> for PathMonitor {
    type Result = ();

    #[tracing::instrument(skip_all, name = "Handler->MultipleInnerChangeEvent->PathMonitor")]
    fn handle(&mut self, msg: MultipleInnerChangeEvent, _ctx: &mut Self::Context) -> Self::Result {
        debug!("  └ got {} events to process", msg.events.len());
        // let now = Instant::now();
        // let original_len = msg.events.len();
        // let unique_len = unique.len();
        let unique = msg.events.iter().collect::<BTreeSet<_>>();
        debug!("  └ {} unique event after converting to set", unique.len());
        debug!("  └ {:?}", unique);
        let outgoing = unique
            .into_iter()
            .filter_map(|e| match &e.kind {
                FsEventKind::Change(pd) => Some(pd.to_owned()),
                _ => None,
            })
            .collect::<Vec<_>>();
        self.sys
            .do_send(FsEventGrouping::buffered_change(outgoing, self.fs_ctx))
    }
}
impl Handler<SingleInnerChangeEvent> for PathMonitor {
    type Result = ();
    #[tracing::instrument(skip_all, name = "Handler->SingleInnerChangeEvent->PathMonitor")]
    fn handle(&mut self, msg: SingleInnerChangeEvent, _ctx: &mut Self::Context) -> Self::Result {
        debug!("will forward single event to sys");
        self.sys.do_send(FsEventGrouping::Singular(msg.event))
    }
}

impl Handler<FsEvent> for PathMonitor {
    type Result = ();
    fn handle(&mut self, msg: FsEvent, _ctx: &mut Self::Context) -> Self::Result {
        let span = debug_span!("Handler->FsEvent->PathMonitor");
        let _guard = span.enter();
        debug!(?self.fs_ctx);
        let sender = self.inner_sender.clone();
        match &msg.kind {
            FsEventKind::Change(PathDescriptionOwned { .. }) => {
                debug!("FsEventKind::Change");
                Arbiter::current().spawn(async move {
                    match sender.send(msg).await {
                        Ok(_) => {}
                        Err(e) => tracing::error!(?e, "could not send"),
                    };
                });
            }
            FsEventKind::PathAdded(PathAddedEvent { path }) => {
                debug!("FsEventKind::PathAdded {}", path.display());
            }
            FsEventKind::PathRemoved(PathEvent { path }) => {
                debug!("FsEventKind::PathRemoved {}", path.display())
            }
            FsEventKind::PathNotFoundError(PathEvent { path }) => {
                debug!("FsEventKind::PathNotFoundError {}", path.display())
            }
        }
    }
}

fn to_watcher(
    cwd: &Path,
    opts: &Spec,
    fs_ctx: FsEventContext,
    receiver: Recipient<FsEvent>,
) -> FsWatcher {
    let mut watcher = FsWatcher::new(cwd, fs_ctx, receiver);

    if let Some(filter_kind) = &opts.filter {
        let filters = convert(filter_kind);
        for filter in filters {
            debug!(filter = ?filter, "append filter");
            watcher.with_filter(filter);
        }
    }
    if let Some(ignore_filter_kind) = &opts.ignore {
        let ignores = convert(ignore_filter_kind);
        for ignore in ignores {
            debug!(ignore = ?ignore, "with ignore");
            watcher.with_ignore(ignore);
        }
    }

    watcher
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct StopPathMonitor;

impl Handler<StopPathMonitor> for PathMonitor {
    type Result = ();

    fn handle(&mut self, _msg: StopPathMonitor, _ctx: &mut Self::Context) -> Self::Result {
        for x in &self.addrs {
            x.do_send(StopWatcher)
        }
        self.addrs = vec![];
    }
}

impl PathMonitor {
    pub fn new(
        sys: Recipient<FsEventGrouping>,
        debounce: Debounce,
        cwd: PathBuf,
        fs_ctx: FsEventContext,
        path_watchable: PathWatchable,
    ) -> Self {
        let (inner_sender, inner_receiver) = mpsc::channel::<FsEvent>(1);
        Self {
            sys,
            debounce,
            cwd,
            addrs: vec![],
            fs_ctx,
            path_watchable,
            inner_sender,
            inner_receiver: Some(inner_receiver),
        }
    }
}
