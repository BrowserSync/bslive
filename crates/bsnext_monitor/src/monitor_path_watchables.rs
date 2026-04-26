use crate::Monitor;
use crate::path_monitor::{PathMonitor, StopPathMonitor};
use crate::path_monitor_meta::PathMonitorMeta;
use crate::watchables::MonitorPathWatchables;
use crate::watchables::path_watchable::PathWatchable;
use actix::{Actor, Addr, AsyncContext, ResponseFuture};
use bsnext_fs::{Debounce, FsEventContext};
use std::collections::BTreeSet;
use tracing::{debug, debug_span};

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct DropMonitor(pub PathWatchable);

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct InsertMonitor(
    pub PathWatchable,
    pub Addr<PathMonitor>,
    pub PathMonitorMeta,
);

impl actix::Handler<MonitorPathWatchables> for Monitor {
    type Result = ResponseFuture<()>;

    #[tracing::instrument(skip_all, name = "MonitorPathWatchables->Monitor")]
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
            let paths = any_watchable
                .watch_paths()
                .into_iter()
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();
            tracing::trace!(?watchable_hash);

            let fs_ctx_id = match any_watchable {
                PathWatchable::Server(watchable) => watchable.server_identity.as_id(),
                PathWatchable::Route(watchable) => watchable.server_identity.as_id(),
                PathWatchable::Any(_) => watchable_hash,
            };
            tracing::trace!(?fs_ctx_id);

            let fs_ctx = FsEventContext::new(fs_ctx_id, watchable_hash);
            let spec = any_watchable.spec();
            tracing::trace!(?spec);

            let debounce = spec.debounce.map(Debounce::from).unwrap_or_default();
            tracing::trace!(?debounce);

            let monitor = PathMonitor::new(
                msg.recipient.clone(),
                debounce,
                msg.cwd.clone(),
                fs_ctx,
                spec.clone(),
                paths,
            );

            let meta = PathMonitorMeta::from(&monitor);
            tracing::trace!(?meta);

            let monitor_addr = monitor.start();

            ctx.notify(InsertMonitor((*any_watchable).clone(), monitor_addr, meta))
        }

        Box::pin(async move {})
    }
}

impl actix::Handler<DropMonitor> for Monitor {
    type Result = ();

    fn handle(&mut self, msg: DropMonitor, _ctx: &mut Self::Context) -> Self::Result {
        tracing::trace!(watchable=?msg.0, "DropMonitor");
        self.any_monitors.remove(&msg.0);
        tracing::trace!("dropped, Monitor count {}", self.any_monitors.len());
    }
}

impl actix::Handler<InsertMonitor> for Monitor {
    type Result = ();

    fn handle(
        &mut self,
        InsertMonitor(watchable, addr, meta): InsertMonitor,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let span = debug_span!("InsertMonitor");
        let _guard = span.enter();
        debug!("{}", watchable);
        self.any_monitors.insert(watchable, (addr, meta));
        debug!("+ Monitor count {}", self.any_monitors.len());
    }
}
