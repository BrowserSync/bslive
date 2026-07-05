use crate::PathMonitorEvent;
use crate::path_and_filter::PathAndFilter;
use crate::watch_paths_msg::WatchPaths;
use actix::{Actor, ActorContext, Addr, AsyncContext, Context, Handler, Recipient, StreamHandler};
use actix_rt::Arbiter;
use bsnext_fs::actor::FsWatcher;
use bsnext_fs::buffered_debounce::BufferedStreamOpsExt;
use bsnext_fs::filter::{Filter, FilterScope};
use bsnext_fs::stop_handler::StopWatcher;
use bsnext_fs::stream::StreamOpsExt;
use bsnext_fs::watch_path_handler::RequestWatchPath;
use bsnext_fs::{Debounce, FsEvent, FsEventContext, FsEventKind, PathDescriptionOwned};
use bsnext_input::route::{PathPattern, WatchSpec};
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tracing::{debug, debug_span};

#[derive(Debug)]
pub struct PathMonitor {
    pub(crate) cwd: PathBuf,
    pub(crate) fs_ctx: FsEventContext,
    pub(crate) debounce: Debounce,
    pub(crate) watch_spec: WatchSpec,
    addrs: Vec<Addr<FsWatcher>>,
    recipient: Recipient<PathMonitorEvent>,
    inner_sender: tokio::sync::mpsc::Sender<FsEvent>,
    inner_receiver: Option<tokio::sync::mpsc::Receiver<FsEvent>>,
}

impl PathMonitor {
    pub fn new(
        recipient: Recipient<PathMonitorEvent>,
        debounce: Debounce,
        cwd: PathBuf,
        fs_ctx: FsEventContext,
        watch_spec: WatchSpec,
    ) -> Self {
        let (inner_sender, inner_receiver) = mpsc::channel::<FsEvent>(1);
        Self {
            recipient,
            debounce,
            cwd,
            addrs: vec![],
            fs_ctx,
            watch_spec,
            inner_sender,
            inner_receiver: Some(inner_receiver),
        }
    }
}

impl actix::Actor for PathMonitor {
    type Context = actix::Context<Self>;
}

impl actix::Handler<WatchPaths> for PathMonitor {
    type Result = ();

    fn handle(&mut self, msg: WatchPaths, ctx: &mut Self::Context) -> Self::Result {
        for single_path in &msg.paths {
            let as_str = single_path.to_string_lossy();
            tracing::debug!(?as_str, "before split");
            let PathAndFilter { path, filter_kind } = PathAndFilter::new(&as_str);
            tracing::debug!(?path, ?filter_kind, "will split");

            // create a filter list, first using the optional filter given above
            let mut filters = filter_kind.into_iter().collect::<Vec<_>>();

            // additional filter from options?
            if let Some(filter) = &self.watch_spec.only {
                filters.push(filter.clone());
            }

            // create the watcher now
            let watcher = to_watcher(
                &self.cwd,
                Some(&PathPattern::List(filters)),
                self.watch_spec.ignore.as_ref(),
                self.fs_ctx,
                ctx.address().recipient(),
            );

            let watcher_addr = watcher.start();

            self.addrs.push(watcher_addr.clone());

            let pb = path.to_path_buf();
            tracing::debug!(?pb, ?path, "as pb");
            watcher_addr.do_send(RequestWatchPath { path: pb });
        }

        let Some(receiver) = self.inner_receiver.take() else {
            panic!("impossible?")
        };

        match self.debounce {
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

impl StreamHandler<FsEvent> for PathMonitor {
    fn handle(&mut self, event: FsEvent, _ctx: &mut Context<PathMonitor>) {
        debug!("StreamHandler<FsEvent> for PathMonitor");
        self.recipient.do_send(PathMonitorEvent::singular(
            event,
            self.watch_spec.clone(),
            self.debounce,
        ))
    }
}

impl StreamHandler<Vec<FsEvent>> for PathMonitor {
    fn handle(&mut self, events: Vec<FsEvent>, _ctx: &mut Context<PathMonitor>) {
        debug!("StreamHandler<Vec<FsEvent>> for PathMonitor");
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
        self.recipient.do_send(PathMonitorEvent::buffered_change(
            outgoing,
            self.fs_ctx,
            self.watch_spec.clone(),
            self.debounce,
        ))
    }
}

pub(crate) fn pattern_to_filter_list(fk: &PathPattern) -> Vec<Filter> {
    match fk {
        PathPattern::StringDefault(string_default) => {
            if string_default.contains("*") {
                let is_abs = Path::new(&string_default).is_absolute();
                let glob = globset::GlobBuilder::new(string_default)
                    .literal_separator(true)
                    .case_insensitive(true)
                    .build()
                    .map(|x| x.compile_matcher());
                match glob {
                    Ok(pattern) if is_abs => vec![Filter::Glob {
                        glob: pattern,
                        raw: string_default.to_string(),
                        scope: FilterScope::Absolute,
                    }],
                    Ok(pattern) => vec![Filter::Glob {
                        glob: pattern,
                        raw: string_default.to_string(),
                        scope: FilterScope::Relative,
                    }],
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
        PathPattern::Extension { ext } => vec![Filter::Extension {
            ext: ext.to_string(),
        }],
        PathPattern::Glob { glob } => {
            let is_abs = Path::new(&glob).is_absolute();
            let matcher = globset::GlobBuilder::new(glob)
                .literal_separator(true)
                .case_insensitive(true)
                .build()
                .map(|x| x.compile_matcher());
            match matcher {
                Ok(pattern) if is_abs => vec![Filter::Glob {
                    glob: pattern,
                    raw: glob.to_string(),
                    scope: FilterScope::Absolute,
                }],
                Ok(pattern) => vec![Filter::Glob {
                    glob: pattern,
                    raw: glob.to_string(),
                    scope: FilterScope::Relative,
                }],
                Err(e) => {
                    tracing::error!("could not use glob '{:?}'", glob);
                    tracing::debug!(?e);
                    vec![]
                }
            }
        }
        PathPattern::List(items) => items
            .iter()
            .flat_map(pattern_to_filter_list)
            .collect::<Vec<_>>(),
        PathPattern::Any { any } => vec![Filter::Any {
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
                debug!("Sending some other event");
                let output =
                    PathMonitorEvent::singular(msg, self.watch_spec.clone(), self.debounce);
                self.recipient.do_send(output)
            }
        }
    }
}

fn to_watcher(
    cwd: &Path,
    filter: Option<&PathPattern>,
    ignore: Option<&PathPattern>,
    fs_ctx: FsEventContext,
    receiver: Recipient<FsEvent>,
) -> FsWatcher {
    let mut watcher = FsWatcher::new(cwd, fs_ctx, receiver);

    if let Some(filter_kind) = &filter {
        let filters = pattern_to_filter_list(filter_kind);
        for filter in filters {
            debug!(filter = %filter, "append filter");
            watcher.with_filter(filter);
        }
    }
    if let Some(ignore_filter_kind) = &ignore {
        let ignores = pattern_to_filter_list(ignore_filter_kind);
        for ignore in ignores {
            debug!(ignore = %ignore, "with ignore");
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
        for addr in &self.addrs {
            addr.do_send(StopWatcher)
        }
        self.addrs = vec![];
        ctx.stop();
    }
}
