use crate::monitor::{AnyWatchable, Monitor};
use crate::BsSystem;
use actix::{Actor, AsyncContext};
use bsnext_fs::actor::FsWatcher;
use bsnext_fs::filter::Filter;
use bsnext_fs::stop_handler::StopWatcher;
use bsnext_fs::watch_path_handler::RequestWatchPath;
use bsnext_fs::Debounce;
use bsnext_input::route::{DebounceDuration, FilterKind, SpecOpts};
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Duration;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct MonitorAnyWatchables {
    pub watchables: Vec<AnyWatchable>,
    pub cwd: PathBuf,
}

impl actix::Handler<MonitorAnyWatchables> for BsSystem {
    type Result = ();

    fn handle(&mut self, msg: MonitorAnyWatchables, ctx: &mut Self::Context) -> Self::Result {
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
                mon.addr.do_send(StopWatcher);
                ctx.notify(DropMonitor((*any_watchable).clone()))
            }
        }

        tracing::debug!("adding {} new watchables", to_add.len());
        for watchable in to_add {
            let mut input_watcher = match watchable {
                AnyWatchable::Server(server_watchable) => {
                    FsWatcher::new(&msg.cwd, server_watchable.server_identity.as_id())
                }
                AnyWatchable::Route(route_watchable) => {
                    FsWatcher::new(&msg.cwd, route_watchable.server_identity.as_id())
                }
            };

            if let Some(opts) = &watchable.spec_opts() {
                // todo: handle the specs here!
                if let Some(filter) = &opts.filter {
                    let f = match filter {
                        FilterKind::Extension { ext } => Filter::Extension {
                            ext: ext.to_string(),
                        },
                        FilterKind::StringGlob(glob) => Filter::Glob {
                            glob: glob.to_owned(),
                        },
                        FilterKind::Glob { glob } => Filter::Glob {
                            glob: glob.to_owned(),
                        },
                    };
                    input_watcher.with_filter(f);
                }
            }

            let duration = match &watchable.spec_opts() {
                Some(SpecOpts {
                    debounce: Some(DebounceDuration::Ms(ms)),
                    ..
                }) => Duration::from_millis(*ms),
                _ => Duration::from_millis(300),
            };

            input_watcher.with_debounce(Debounce::Buffered { duration });

            let input_watcher_addr = input_watcher.start();

            let monitor = Monitor {
                addr: input_watcher_addr.clone(),
                path: watchable.watch_path().to_path_buf(),
            };

            monitor.addr.do_send(RequestWatchPath {
                recipients: vec![ctx.address().recipient()],
                path: monitor.path.clone(),
            });

            ctx.notify(InsertMonitor((*watchable).clone(), monitor))
        }
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
struct DropMonitor(AnyWatchable);

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
struct InsertMonitor(AnyWatchable, Monitor);

impl actix::Handler<InsertMonitor> for BsSystem {
    type Result = ();

    fn handle(&mut self, msg: InsertMonitor, _ctx: &mut Self::Context) -> Self::Result {
        tracing::trace!(watchable=?msg.0, "InsertMonitor");
        self.any_monitors.insert(msg.0, msg.1);
        tracing::trace!("inserted, Monitor count {}", self.any_monitors.len());
    }
}
