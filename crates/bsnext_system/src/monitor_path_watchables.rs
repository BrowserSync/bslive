use crate::path_monitor::{PathMonitor, PathMonitorMeta, StopPathMonitor};
use crate::path_watchable::PathWatchable;
use crate::BsSystem;
use actix::{Actor, Addr, AsyncContext};
use bsnext_fs::{Debounce, FsEventContext};
use bsnext_input::route::{DebounceDuration, Spec};
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
            if let Some((path_monitor_addr, _)) = self.any_monitors.get(any_watchable) {
                path_monitor_addr.do_send(StopPathMonitor);
                ctx.notify(DropMonitor((*any_watchable).clone()))
            }
        }

        for (index, any_watchable) in to_add.into_iter().enumerate() {
            let span = debug_span!("{}", index);
            let _guard = span.enter();
            let watchable_hash = any_watchable.as_id();
            tracing::trace!(?watchable_hash);

            let fs_ctx_id = match any_watchable {
                PathWatchable::Server(watchable) => watchable.server_identity.as_id(),
                PathWatchable::Route(watchable) => watchable.server_identity.as_id(),
                PathWatchable::Any(_) => watchable_hash,
            };
            tracing::trace!(?fs_ctx_id);

            let fs_ctx = FsEventContext::new(fs_ctx_id, watchable_hash);
            let opts = any_watchable.spec_opts();
            tracing::trace!(?opts);

            let duration = match opts {
                Spec {
                    debounce: Some(DebounceDuration::Ms(ms)),
                    ..
                } => Duration::from_millis(*ms),
                _ => Duration::from_millis(300),
            };
            tracing::trace!(?duration);

            let debounce = Debounce::Buffered { duration };
            tracing::trace!(?debounce);

            let monitor = PathMonitor::new(
                ctx.address().recipient(),
                debounce,
                msg.cwd.clone(),
                fs_ctx,
                (*any_watchable).clone(),
            );

            let meta = PathMonitorMeta::from(&monitor);
            tracing::trace!(?meta);

            let monitor = monitor.start();

            ctx.notify(InsertMonitor((*any_watchable).clone(), monitor, meta))
        }
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
struct InsertMonitor(PathWatchable, Addr<PathMonitor>, PathMonitorMeta);

impl actix::Handler<InsertMonitor> for BsSystem {
    type Result = ();

    fn handle(&mut self, msg: InsertMonitor, _ctx: &mut Self::Context) -> Self::Result {
        let span = debug_span!("InsertMonitor");
        let _guard = span.enter();
        debug!("{}", msg.0);
        self.any_monitors.insert(msg.0, (msg.1, msg.2));
        debug!("+ Monitor count {}", self.any_monitors.len());
    }
}
