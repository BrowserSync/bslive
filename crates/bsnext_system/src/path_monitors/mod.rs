use crate::watchables::path_watchable::PathWatchable;
use crate::watchables::MonitorPathWatchables;
use actix::{Actor, Addr, AsyncContext, ResponseFuture};
use bsnext_fs::{Debounce, FsEventContext};
use bsnext_path_monitor::path_monitor::{PathMonitor, StopPathMonitor};
use bsnext_path_monitor::watch_paths_msg::WatchPaths;
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use tracing::{debug, debug_span};

#[derive(Debug, Default)]
pub struct PathMonitors {
    path_monitors: HashMap<PathWatchable, Addr<PathMonitor>>,
}

impl PathMonitors {
    pub fn new() -> Self {
        Self {
            path_monitors: Default::default(),
        }
    }
}

impl actix::Actor for PathMonitors {
    type Context = actix::Context<Self>;
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct DropMonitor(pub PathWatchable);

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct InsertMonitor(pub PathWatchable, pub Addr<PathMonitor>, pub Vec<PathBuf>);

impl actix::Handler<MonitorPathWatchables> for PathMonitors {
    type Result = ResponseFuture<()>;

    #[tracing::instrument(skip_all, name = "MonitorPathWatchables->Monitor")]
    fn handle(&mut self, msg: MonitorPathWatchables, ctx: &mut Self::Context) -> Self::Result {
        debug!("{}", file!());

        let existing = self.path_monitors.keys().collect::<BTreeSet<_>>();
        let incoming = msg.watchables.iter().collect::<BTreeSet<_>>();

        let in_both = incoming.intersection(&existing).collect::<Vec<_>>();
        let to_add = incoming.difference(&existing).collect::<Vec<_>>();
        let to_remove = existing.difference(&incoming).collect::<Vec<_>>();

        debug!("{} duplicates", in_both.len());
        debug!("{} monitors to remove", to_remove.len());
        debug!("{} monitors to add", to_add.len());

        for any_watchable in to_remove {
            if let Some(path_monitor_addr) = self.path_monitors.get(any_watchable) {
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

            let path_monitor = PathMonitor::new(
                msg.recipient.clone(),
                debounce,
                msg.cwd.clone(),
                fs_ctx,
                spec.clone(),
            );

            let path_monitor_addr = path_monitor.start();

            ctx.notify(InsertMonitor(
                (*any_watchable).clone(),
                path_monitor_addr,
                paths,
            ))
        }

        Box::pin(async move {})
    }
}

impl actix::Handler<DropMonitor> for PathMonitors {
    type Result = ();

    fn handle(&mut self, msg: DropMonitor, _ctx: &mut Self::Context) -> Self::Result {
        tracing::trace!(watchable=?msg.0, "DropMonitor");
        self.path_monitors.remove(&msg.0);
        tracing::trace!("dropped, Monitor count {}", self.path_monitors.len());
    }
}

impl actix::Handler<InsertMonitor> for PathMonitors {
    type Result = ();

    fn handle(
        &mut self,
        InsertMonitor(watchable, addr, paths): InsertMonitor,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let span = debug_span!("InsertMonitor");
        let _guard = span.enter();
        debug!("{}", watchable);
        self.path_monitors.insert(watchable, addr.clone());
        debug!("+ Monitor count {}", self.path_monitors.len());
        addr.do_send(WatchPaths { paths });
    }
}
