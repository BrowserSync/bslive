use crate::watchables::path_watchable::PathWatchable;
use crate::watchables::MonitorPathWatchables;
use actix::ActorFutureExt;
use actix::{Actor, Addr, AsyncContext, Recipient, ResponseActFuture, ResponseFuture, WrapFuture};
use bsnext_dto::status_events::{MonitorStatusReader, MonitorStatusWriter, ServiceStatus};
use bsnext_fs::{Debounce, FsEventContext};
use bsnext_path_monitor::path_monitor::{PathMonitor, StopPathMonitor};
use bsnext_path_monitor::watch_paths_msg::WatchPaths;
use bsnext_path_monitor::PathMonitorChangeset;
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use tracing::{debug, debug_span};

#[derive(Debug)]
pub struct PathMonitors {
    cwd: PathBuf,
    path_monitors: HashMap<PathWatchable, Addr<PathMonitor>>,
    state_reader: Recipient<MonitorStatusReader>,
    state_writer: Recipient<MonitorStatusWriter>,
    status: ServiceStatus,
    recipient: Option<Recipient<PathMonitorChangeset>>,
}

impl PathMonitors {
    pub fn new(
        cwd: PathBuf,
        state_reader: Recipient<MonitorStatusReader>,
        state_writer: Recipient<MonitorStatusWriter>,
    ) -> Self {
        Self {
            cwd,
            path_monitors: Default::default(),
            state_writer,
            state_reader,
            status: ServiceStatus::default(),
            recipient: None,
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

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct RequestMonitor {
    inner: MonitorPathWatchables,
    recipient: Recipient<PathMonitorChangeset>,
    cwd: PathBuf,
}

impl RequestMonitor {
    pub(crate) fn new(
        inner: MonitorPathWatchables,
        recipient: Recipient<PathMonitorChangeset>,
        cwd: PathBuf,
    ) -> Self {
        Self {
            inner,
            recipient,
            cwd,
        }
    }
}

impl actix::Handler<RequestMonitor> for PathMonitors {
    type Result = ResponseFuture<()>;

    #[tracing::instrument(skip_all, name = "RequestMonitor->Monitor")]
    fn handle(
        &mut self,
        RequestMonitor {
            inner,
            recipient,
            cwd,
        }: RequestMonitor,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        debug!("{}", file!());

        let existing = self.path_monitors.keys().collect::<BTreeSet<_>>();
        let incoming = inner.watchables.iter().collect::<BTreeSet<_>>();

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
                recipient.clone(),
                debounce,
                cwd.clone(),
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

/// Message to monitor file system paths based on the current input configuration.
///
/// This handler processes an `Input` configuration to determine which paths should be
/// monitored for file system changes. It:
/// - Converts the input into a collection of watchables based on the configured watch strategy
///   (routes, servers, or any custom watchers)
/// - Sends these watchables to the `Monitor` actor to create individual `PathMonitor` instances
/// - Registers the resulting task specifications with the system for later use in handling
///   file system events
///
/// The monitoring configuration respects the `watchers` and `infer` settings in the input,
/// allowing for flexible control over which files and directories are watched.
#[derive(actix::Message)]
#[rtype(result = "Result<(), anyhow::Error>")]
pub struct MonitorAny;

impl Default for MonitorAny {
    fn default() -> Self {
        Self::new()
    }
}

impl MonitorAny {
    pub fn new() -> Self {
        Self
    }
}

impl actix::Handler<MonitorAny> for PathMonitors {
    type Result = ResponseActFuture<Self, Result<(), anyhow::Error>>;

    fn handle(&mut self, _msg: MonitorAny, ctx: &mut Self::Context) -> Self::Result {
        let cwd = self.cwd.clone();
        let monitors_addr = ctx.address();
        let state_reader = self.state_reader.clone();
        let local_status = self.status;
        let Some(recipient) = self.recipient.clone() else {
            todo!("How can this happen?");
        };
        //
        let future = async move {
            let Ok((sys_service_status, input)) = state_reader.send(MonitorStatusReader).await
            else {
                unreachable!("can it fail?")
            };
            if let (ServiceStatus::Ready { .. }, ServiceStatus::Ready { .. }) =
                (sys_service_status, local_status)
            {
                tracing::info!("doing nothing, sys + local state match");
                tracing::info!(?sys_service_status);
                tracing::info!(?local_status);

                return Ok(None);
            }

            tracing::info!("There's something to reconcile...");

            let expected_id = match sys_service_status {
                ServiceStatus::Idle => todo!("how can we get here?"),
                ServiceStatus::Reconciling { desired, .. } => desired,
                ServiceStatus::Ready { .. } => todo!("how...?"),
            };

            let watchables = MonitorPathWatchables::new(&input);
            let request = RequestMonitor::new(watchables, recipient, cwd);
            monitors_addr.send(request).await?;
            Ok::<_, anyhow::Error>(Some(expected_id))
        };

        Box::pin(future.into_actor(self).map(|res, actor, _ctx| {
            match res {
                Ok(None) => {
                    tracing::info!("got None, returning with nothing to do...?");
                    return Ok(());
                }
                Ok(Some(desired)) => {
                    // todo - make sure we can handle NOT being ready...
                    actor.status = ServiceStatus::Ready {
                        desired,
                        observed: desired,
                    };
                    let state_writer = actor.state_writer.clone();

                    state_writer.do_send(MonitorStatusWriter {
                        monitor_status: actor.status,
                    });
                }
                Err(err) => {
                    tracing::error!(?err);
                }
            }
            Ok(())
        }))
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct ConfigureRecipient {
    recipient: Recipient<PathMonitorChangeset>,
}

impl ConfigureRecipient {
    pub fn new(recipient: Recipient<PathMonitorChangeset>) -> Self {
        Self { recipient }
    }
}

impl actix::Handler<ConfigureRecipient> for PathMonitors {
    type Result = ();

    fn handle(&mut self, msg: ConfigureRecipient, _ctx: &mut Self::Context) -> Self::Result {
        self.recipient = Some(msg.recipient);
    }
}
