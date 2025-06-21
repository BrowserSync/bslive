use crate::path_watchable::PathWatchable;
use actix::{ActorContext, Addr, AsyncContext, Context, Handler, Recipient, StreamHandler};
use actix_rt::Arbiter;
use bsnext_fs::actor::FsWatcher;
use bsnext_fs::buffered_debounce::BufferedStreamOpsExt;
use bsnext_fs::filter::Filter;
use bsnext_fs::stop_handler::StopWatcher;
use bsnext_fs::stream::StreamOpsExt;
use bsnext_fs::watch_path_handler::RequestWatchPath;
use bsnext_fs::{
    Debounce, FsEvent, FsEventContext, FsEventGrouping, FsEventKind, PathDescriptionOwned,
};
use bsnext_input::route::FilterKind;
use glob::Pattern;
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

#[derive(Debug, Clone)]
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

impl StreamHandler<FsEvent> for PathMonitor {
    fn handle(&mut self, event: FsEvent, _ctx: &mut Context<PathMonitor>) {
        self.sys.do_send(FsEventGrouping::Singular(event))
    }
}

impl StreamHandler<Vec<FsEvent>> for PathMonitor {
    fn handle(&mut self, events: Vec<FsEvent>, _ctx: &mut Context<PathMonitor>) {
        debug!("  └ got {} events to process", events.len());
        // let now = Instant::now();
        // let original_len = events.len();
        // let unique_len = unique.len();
        let unique = events.iter().collect::<BTreeSet<_>>();
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

impl actix::Actor for PathMonitor {
    type Context = actix::Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        for single_path in &self.path_watchable.watch_paths() {
            let as_str = single_path.to_string_lossy();
            let (path, filter) = path_and_filter_p(&as_str);

            // create a filter list, first using the optional filter given above
            let mut filters = filter.into_iter().collect::<Vec<_>>();

            // additional filter from options?
            let spec_opts = self.path_watchable.spec_opts();
            if let Some(filter) = &spec_opts.filter {
                filters.push(filter.clone());
            }

            // create the watcher now
            let watcher = to_watcher(
                &self.cwd,
                Some(&FilterKind::List(filters)),
                spec_opts.filter.as_ref(),
                self.fs_ctx,
                ctx.address().recipient(),
            );

            let watcher_addr = watcher.start();
            let watcher_addr_clone = watcher_addr.clone();

            self.addrs.push(watcher_addr);

            watcher_addr_clone.do_send(RequestWatchPath {
                path: path.to_path_buf(),
            });
        }
        let Some(receiver) = self.inner_receiver.take() else {
            panic!("impossible?")
        };
        let debounce = self.debounce;
        match debounce {
            Debounce::Trailing { duration } => {
                let stream = ReceiverStream::new(receiver).debounce(duration);
                <Self as StreamHandler<FsEvent>>::add_stream(stream, ctx);
            }
            Debounce::Buffered { duration } => {
                let stream = ReceiverStream::new(receiver).buffered_debounce(duration);
                <Self as StreamHandler<Vec<FsEvent>>>::add_stream(stream, ctx);
            }
        }
    }
}

fn path_and_filter_p(p: &str) -> (&Path, Option<FilterKind>) {
    if let Some((before, ..)) = p.split_once("*") {
        (
            Path::new(before),
            Some(FilterKind::Glob {
                glob: p.to_string(),
            }),
        )
    } else {
        (Path::new(p), None)
    }
}

fn convert(fk: &FilterKind) -> Vec<Filter> {
    match fk {
        FilterKind::StringDefault(string_default) => {
            if string_default.contains("*") {
                let pattern = Pattern::new(string_default);
                match pattern {
                    Ok(pattern) => {
                        vec![Filter::Glob { glob: pattern }]
                    }
                    Err(e) => {
                        tracing::error!("could not use glob {:?}", string_default);
                        tracing::debug!(?e);
                        vec![]
                    }
                }
            } else {
                vec![Filter::Any {
                    any: string_default.to_string(),
                }]
            }
        }
        FilterKind::Extension { ext } => vec![Filter::Extension {
            ext: ext.to_string(),
        }],
        FilterKind::Glob { glob } => {
            let pattern = Pattern::new(glob);
            match pattern {
                Ok(pattern) => vec![Filter::Glob { glob: pattern }],
                Err(e) => {
                    tracing::error!("could not use glob {:?}", glob);
                    tracing::debug!(?e);
                    vec![]
                }
            }
        }
        FilterKind::List(items) => items.iter().flat_map(convert).collect::<Vec<_>>(),
        FilterKind::Any { any } => vec![Filter::Any {
            any: any.to_string(),
        }],
    }
}

impl Handler<FsEvent> for PathMonitor {
    type Result = ();
    fn handle(&mut self, msg: FsEvent, _ctx: &mut Self::Context) -> Self::Result {
        let span = debug_span!("Handler->FsEvent->PathMonitor");
        let _guard = span.enter();
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
            _ => {
                // todo: any need to buffer these?
                self.sys.do_send(FsEventGrouping::Singular(msg))
            }
        }
    }
}

fn to_watcher(
    cwd: &Path,
    filter: Option<&FilterKind>,
    ignore: Option<&FilterKind>,
    fs_ctx: FsEventContext,
    receiver: Recipient<FsEvent>,
) -> FsWatcher {
    let mut watcher = FsWatcher::new(cwd, fs_ctx, receiver);

    if let Some(filter_kind) = &filter {
        let filters = convert(filter_kind);
        for filter in filters {
            debug!(filter = ?filter, "append filter");
            watcher.with_filter(filter);
        }
    }
    if let Some(ignore_filter_kind) = &ignore {
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

    fn handle(&mut self, _msg: StopPathMonitor, ctx: &mut Self::Context) -> Self::Result {
        for x in &self.addrs {
            x.do_send(StopWatcher)
        }
        self.addrs = vec![];
        ctx.stop();
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
