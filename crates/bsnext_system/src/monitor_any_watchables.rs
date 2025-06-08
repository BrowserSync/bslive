use crate::path_monitor::PathMonitor;
use crate::path_watchable::PathWatchable;
use crate::BsSystem;
use actix::{Actor, AsyncContext};
use bsnext_fs::actor::FsWatcher;
use bsnext_fs::filter::Filter;
use bsnext_fs::stop_handler::StopWatcher;
use bsnext_fs::watch_path_handler::RequestWatchPath;
use bsnext_fs::{Debounce, FsEventContext};
use bsnext_input::route::{DebounceDuration, FilterKind, Spec};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, debug_span};

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct MonitorPathWatchables {
    pub watchables: Vec<PathWatchable>,
    pub cwd: PathBuf,
}

impl actix::Handler<MonitorPathWatchables> for BsSystem {
    type Result = ();

    #[tracing::instrument(skip_all, name = "Handler->MonitorPathWatchables->BsSystem")]
    fn handle(&mut self, msg: MonitorPathWatchables, ctx: &mut Self::Context) -> Self::Result {
        debug!("{}", file!());

        let existing = self.any_monitors.keys().collect::<BTreeSet<_>>();
        let incoming = msg.watchables.iter().collect::<BTreeSet<_>>();

        let in_both = incoming.intersection(&existing).collect::<Vec<_>>();
        let to_add = incoming.difference(&existing).collect::<Vec<_>>();
        let to_remove = existing.difference(&incoming).collect::<Vec<_>>();

        debug!("{} duplicates", in_both.len());
        debug!("{} monitors to remove", to_remove.len());
        debug!("{} monitors to add", to_add.len());

        for any_watchable in to_remove {
            if let Some(mon) = self.any_monitors.get(any_watchable) {
                mon.addr.do_send(StopWatcher);
                ctx.notify(DropMonitor((*any_watchable).clone()))
            }
        }

        for (index, any_watchable) in to_add.into_iter().enumerate() {
            let span = debug_span!("{}", index);
            let _guard = span.enter();
            let watchable_hash = any_watchable.as_id();

            let fs_ctx_id = match any_watchable {
                PathWatchable::Server(watchable) => watchable.server_identity.as_id(),
                PathWatchable::Route(watchable) => watchable.server_identity.as_id(),
                PathWatchable::Any(_) => watchable_hash,
            };

            let fs_ctx = FsEventContext::new(fs_ctx_id, watchable_hash);
            let mut watcher = FsWatcher::new(&msg.cwd, fs_ctx.clone());

            let opts = any_watchable.spec_opts();

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

            let duration = match opts {
                Spec {
                    debounce: Some(DebounceDuration::Ms(ms)),
                    ..
                } => Duration::from_millis(*ms),
                _ => Duration::from_millis(300),
            };

            let debounce = Debounce::Buffered { duration };
            watcher.with_debounce(debounce);

            debug!("{}", watcher);

            let watcher_addr = watcher.start();
            let monitor = PathMonitor::new(watcher_addr, fs_ctx);

            for single_path in &any_watchable.watch_paths() {
                debug!(path = %single_path.display());
                monitor.addr.do_send(RequestWatchPath {
                    recipients: vec![ctx.address().recipient()],
                    path: single_path.to_path_buf(),
                });
            }

            ctx.notify(InsertMonitor((*any_watchable).clone(), monitor))
        }
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

#[derive(actix::Message)]
#[rtype(result = "()")]
struct DropMonitor(PathWatchable);

impl actix::Handler<DropMonitor> for BsSystem {
    type Result = ();

    fn handle(&mut self, msg: DropMonitor, _ctx: &mut Self::Context) -> Self::Result {
        tracing::trace!(watchable=?msg.0, "DropMonitor");
        self.any_monitors.remove(&msg.0);
        tracing::trace!("dropped, Monitor count {}", self.any_monitors.len());
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
struct InsertMonitor(PathWatchable, PathMonitor);

impl actix::Handler<InsertMonitor> for BsSystem {
    type Result = ();

    fn handle(&mut self, msg: InsertMonitor, _ctx: &mut Self::Context) -> Self::Result {
        let span = debug_span!("InsertMonitor");
        let _guard = span.enter();
        debug!("{}", msg.0);
        self.any_monitors.insert(msg.0, msg.1);
        debug!("+ Monitor count {}", self.any_monitors.len());
    }
}
