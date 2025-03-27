use crate::any_monitor::{AnyMonitor, PathWatchable};
use crate::path_monitor::PathMonitor;
use crate::BsSystem;
use actix::{Actor, AsyncContext};
use bsnext_fs::actor::FsWatcher;
use bsnext_fs::filter::Filter;
use bsnext_fs::stop_handler::StopWatcher;
use bsnext_fs::watch_path_handler::RequestWatchPath;
use bsnext_fs::{Debounce, FsEventContext};
use bsnext_input::route::{DebounceDuration, FilterKind, SpecOpts};
use std::collections::BTreeSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::path::PathBuf;
use std::time::Duration;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct MonitorPathWatchables {
    pub watchables: Vec<PathWatchable>,
    pub cwd: PathBuf,
}

impl actix::Handler<MonitorPathWatchables> for BsSystem {
    type Result = ();

    fn handle(&mut self, msg: MonitorPathWatchables, ctx: &mut Self::Context) -> Self::Result {
        tracing::debug!("MonitorAnyWatchables {:?}", msg.watchables);
        tracing::trace!("MonitorAnyWatchables {:#?}", msg.watchables);

        let existing = self.any_monitors.keys().collect::<BTreeSet<_>>();
        let incoming = msg.watchables.iter().collect::<BTreeSet<_>>();

        let in_both = incoming.intersection(&existing).collect::<Vec<_>>();
        let to_add = incoming.difference(&existing).collect::<Vec<_>>();
        let to_remove = existing.difference(&incoming).collect::<Vec<_>>();

        tracing::debug!("{} watchables exist in current + incoming", in_both.len());
        tracing::debug!("removing {} watchables", to_remove.len());

        for any_watchable in to_remove {
            if let Some(mon) = self.any_monitors.get(any_watchable) {
                mon.fs_addr().do_send(StopWatcher);
                ctx.notify(DropMonitor((*any_watchable).clone()))
            }
        }

        tracing::debug!("adding {} new watchables", to_add.len());
        for any_watchable in to_add {
            let mut hasher = DefaultHasher::new();
            any_watchable.hash(&mut hasher);
            let watchable_hash = hasher.finish();

            let mut watcher = match any_watchable {
                PathWatchable::Server(server_watchable) => {
                    let id = server_watchable.server_identity.as_id();
                    let ctx = FsEventContext {
                        id,
                        origin_id: watchable_hash,
                    };
                    FsWatcher::new(&msg.cwd, ctx)
                }
                PathWatchable::Route(route_watchable) => {
                    let id = route_watchable.server_identity.as_id();
                    let ctx = FsEventContext {
                        id,
                        origin_id: watchable_hash,
                    };
                    FsWatcher::new(&msg.cwd, ctx)
                }
            };

            if let Some(opts) = &any_watchable.spec_opts() {
                if let Some(filter_kind) = &opts.filter {
                    let filters = convert(filter_kind);
                    for filter in filters {
                        watcher.with_filter(filter);
                    }
                }
                if let Some(ignore_filter_kind) = &opts.ignore {
                    let ignores = convert(ignore_filter_kind);
                    for ignore in ignores {
                        watcher.with_ignore(ignore);
                    }
                }
            }

            let duration = match &any_watchable.spec_opts() {
                Some(SpecOpts {
                    debounce: Some(DebounceDuration::Ms(ms)),
                    ..
                }) => Duration::from_millis(*ms),
                _ => Duration::from_millis(300),
            };

            watcher.with_debounce(Debounce::Buffered { duration });

            let input_watcher_addr = watcher.start();

            let monitor = PathMonitor {
                addr: input_watcher_addr.clone(),
                path: any_watchable.watch_path().to_path_buf(),
                watchable_hash,
            };

            monitor.addr.do_send(RequestWatchPath {
                recipients: vec![ctx.address().recipient()],
                path: monitor.path.clone(),
            });

            let any = AnyMonitor::Path(monitor);

            ctx.notify(InsertMonitor((*any_watchable).clone(), any))
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
struct InsertMonitor(PathWatchable, AnyMonitor);

impl actix::Handler<InsertMonitor> for BsSystem {
    type Result = ();

    fn handle(&mut self, msg: InsertMonitor, _ctx: &mut Self::Context) -> Self::Result {
        tracing::trace!(watchable=?msg.0, "InsertMonitor");
        self.any_monitors.insert(msg.0, msg.1);
        tracing::trace!("inserted, Monitor count {}", self.any_monitors.len());
    }
}
