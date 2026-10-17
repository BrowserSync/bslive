use actix::{Actor, Addr, AsyncContext, Recipient, ResponseActFuture, ResponseFuture, WrapFuture};
use actix::{ActorFutureExt, Running};
use bsnext_dto::status_events::{
    list_hash, MonitorStatusReader, MonitorStatusWriter, ResourceStatus, ServiceStatus, StateValue,
};
use bsnext_fs::{Debounce, FsEventContext};
use bsnext_input::watchables::path_watchable::PathWatchable;
use bsnext_input::watchables::PathWatchables;
use bsnext_input::Input;
use bsnext_path_monitor::path_monitor::{ConnectedWatchers, PathMonitor, StopPathMonitor};
use bsnext_path_monitor::watch_paths_msg::WatchPaths;
use bsnext_path_monitor::PathMonitorChangeset;
use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use std::time::Duration;
use tracing::{debug, debug_span, Instrument};

#[derive(Debug, Default, Clone)]
enum LocalResource {
    #[default]
    Pending,
    Waiting {
        reason: String,
        attempts: usize,
    },
    Ready,
    Failed {
        reason: String,
    },
}

impl From<&'_ LocalResource> for ResourceStatus {
    fn from(value: &'_ LocalResource) -> Self {
        match value {
            LocalResource::Pending => ResourceStatus::Pending,
            LocalResource::Waiting { reason, .. } => ResourceStatus::Waiting {
                reason: reason.clone(),
            },
            LocalResource::Ready => ResourceStatus::Ready,
            LocalResource::Failed { reason } => ResourceStatus::Failed {
                reason: reason.clone(),
            },
        }
    }
}

#[derive(Debug, Default)]
struct LocalState {
    inner: HashMap<PathWatchable, LocalResource>,
}

impl LocalState {
    pub(crate) fn as_resource_status(&self) -> HashMap<PathWatchable, ResourceStatus> {
        let mut resource_status = HashMap::new();
        for (path_watchable, local_resource) in &self.inner {
            resource_status.insert(path_watchable.clone(), local_resource.into());
        }
        resource_status
    }
}

#[derive(Debug)]
pub struct PathMonitors {
    cwd: PathBuf,
    path_monitors: HashMap<PathWatchable, Addr<PathMonitor>>,
    local_state: LocalState,
    state_reader: Recipient<MonitorStatusReader>,
    state_writer: Recipient<MonitorStatusWriter>,
    status: ServiceStatus,
    status_hashes: Vec<u64>,
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
            local_state: Default::default(),
            state_writer,
            state_reader,
            status_hashes: vec![],
            status: ServiceStatus::default(),
            recipient: None,
        }
    }
}

impl actix::Actor for PathMonitors {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(actor.name = "PathMonitors", actor.lifecyle = "started");
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        tracing::trace!(actor.name = "PathMonitors", actor.lifecyle = "stopping");
        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(actor.name = "PathMonitors", actor.lifecyle = "stopped");
        self.local_state.inner.clear();
        todo!("we cleared all local state here...")
    }
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
    inner: PathWatchables,
    recipient: Recipient<PathMonitorChangeset>,
    cwd: PathBuf,
}

impl RequestMonitor {
    pub(crate) fn new(
        inner: PathWatchables,
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
        let to_skip = self
            .local_state
            .inner
            .iter()
            .filter_map(|(k, v)| match v {
                LocalResource::Failed { .. } => Some(k.clone()),
                _ => None,
            })
            .collect::<BTreeSet<_>>();
        let to_add = incoming
            .difference(&existing)
            .filter(|x| !to_skip.contains(x))
            .collect::<Vec<_>>();
        let to_remove = existing.difference(&incoming).collect::<Vec<_>>();

        debug!("{} duplicates", in_both.len());
        debug!("{} monitors to remove", to_remove.len());
        debug!("{} monitors to add", to_add.len());
        debug!("{} monitors to skip", to_skip.len());

        for any_watchable in to_remove {
            if let Some(path_monitor_addr) = self.path_monitors.get(any_watchable) {
                path_monitor_addr.do_send(StopPathMonitor);
                ctx.notify(DropMonitor((*any_watchable).clone()))
            }
        }

        for (index, path_watchable) in to_add.into_iter().enumerate() {
            let span = debug_span!("{}", index);
            let _guard = span.enter();
            let watchable_hash = path_watchable.as_id();
            let paths = path_watchable
                .watch_paths()
                .into_iter()
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();
            tracing::trace!(?watchable_hash);

            let fs_ctx_id = match path_watchable {
                PathWatchable::Server(watchable) => watchable.server_identity.as_id(),
                PathWatchable::Route(watchable) => watchable.server_identity.as_id(),
                PathWatchable::Any(_) => watchable_hash,
            };
            tracing::trace!(?fs_ctx_id);

            let fs_ctx = FsEventContext::new(fs_ctx_id, watchable_hash);
            let spec = path_watchable.spec();
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
            if let Some(matching_state) = self.local_state.inner.get(path_watchable) {
                tracing::debug!(?matching_state)
            }
            ctx.notify(InsertMonitor(
                (**path_watchable).clone(),
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
        let _cwd = self.cwd.clone();
        let monitors_addr = ctx.address();
        let state_reader = self.state_reader.clone();
        let local_status = self.status;
        let _local_hashes = self.status_hashes.clone();
        let Some(_recipient) = self.recipient.clone() else {
            todo!("How can this happen?");
        };
        let future = async move {
            let Ok(..) = monitors_addr.send(TryEvict).await else {
                unreachable!("can it fail?")
            };
            let Ok((sys_service_status, input)) = state_reader.send(MonitorStatusReader).await
            else {
                unreachable!("can it fail?")
            };

            tracing::info!(?sys_service_status, "incoming");
            tracing::info!(?local_status);

            if let (ServiceStatus::Ready { .. }, ServiceStatus::Ready { .. }) =
                (sys_service_status, local_status)
            {
                return Ok(None);
            }
            let expected_id = match sys_service_status {
                ServiceStatus::Idle => todo!("how can we get here?"),
                ServiceStatus::Reconciling { desired, .. } => desired,
                ServiceStatus::Ready { observed, .. } => observed,
            };

            Ok::<_, anyhow::Error>(Some((expected_id, input)))
        };

        Box::pin(future.into_actor(self).map(|res, actor, ctx| {
            let Ok(res) = res else {
                tracing::error!("how did it get here? {:?}", res.unwrap_err());
                return Ok(());
            };
            match res {
                None => {
                    tracing::debug!("--- we decided there's nothing to do because local and remote states match")
                }
                Some((desired, input)) => {
                    tracing::debug!("+++ There's something to reconcile...");
                    actor.status = ServiceStatus::Reconciling {
                        desired,
                        observed: actor.status.observed(),
                    };
                    ctx.notify(TryReconcile { input });
                }
            }
            Ok::<_, anyhow::Error>(())
        }))
    }
}

#[derive(actix::Message)]
#[rtype(result = "Result<(), anyhow::Error>")]
struct TryReconcile {
    input: Input,
}

impl actix::Handler<TryReconcile> for PathMonitors {
    type Result = ResponseActFuture<Self, Result<(), anyhow::Error>>;

    fn handle(&mut self, msg: TryReconcile, ctx: &mut Self::Context) -> Self::Result {
        let input = msg.input;
        let Some(recipient) = self.recipient.clone() else {
            todo!("How can this happen?");
        };
        let monitors_addr = ctx.address();
        let cwd = self.cwd.clone();
        let watchables = PathWatchables::new(&input);
        for x in &watchables.watchables {
            self.local_state.inner.entry(x.clone()).or_default();
        }
        self.state_writer.do_send(MonitorStatusWriter {
            monitor_status: self.status,
            monitor_resource_status: self.local_state.as_resource_status(),
        });
        let states = self.local_state.inner.clone();
        Box::pin(
            async move {
                let span = tracing::debug_span!("try reconcile");
                let _enter = span.enter();
                tracing::debug!(?states);
                let request = RequestMonitor::new(watchables, recipient, cwd);
                monitors_addr.send(request).await?;
                Ok::<_, anyhow::Error>(())
            }
            .in_current_span()
            .into_actor(self)
            .map(|_result, _actor, ctx| {
                ctx.notify(TryRead);
                Ok(())
            }),
        )
    }
}

#[derive(actix::Message)]
#[rtype(result = "Result<(), anyhow::Error>")]
struct TryRead;

impl actix::Handler<TryRead> for PathMonitors {
    type Result = ResponseActFuture<Self, Result<(), anyhow::Error>>;

    fn handle(&mut self, _msg: TryRead, _ctx: &mut Self::Context) -> Self::Result {
        let jobs = self
            .path_monitors
            .clone()
            .iter()
            .map(|(path_watchable, v)| {
                (
                    path_watchable.clone(),
                    (
                        v.clone(),
                        self.local_state
                            .inner
                            .get(path_watchable)
                            .map(ToOwned::to_owned)
                            .expect("If we get here there's a buug"),
                    ),
                )
            })
            .collect::<HashMap<PathWatchable, (Addr<PathMonitor>, LocalResource)>>();
        let jobs_len = jobs.len();
        let state_writer = self.state_writer.clone();

        #[derive(Debug)]
        enum LocalResult {
            Ready {
                observed_status: StateValue,
                local_states: HashMap<PathWatchable, LocalResource>,
            },
            NotReady {
                observed_status: StateValue,
                local_states: HashMap<PathWatchable, LocalResource>,
            },
        }

        let future = async move {
            tracing::debug!(
                "will ask {} addresses for their ConnectedWatchers",
                jobs_len
            );
            let mut hashes = vec![];
            let mut local_states: HashMap<PathWatchable, LocalResource> = HashMap::new();
            for (path_watchable, (addr, local_resource)) in jobs {
                let hash = path_watchable.as_id();
                let re = addr.send(ConnectedWatchers).await?;
                let is_good = re.observed_len == re.desired_len;
                if is_good {
                    hashes.push(hash);
                    local_states.insert(path_watchable, LocalResource::Ready);
                } else {
                    let next = match local_resource {
                        LocalResource::Pending => LocalResource::Waiting {
                            reason: "didn't start".to_string(),
                            attempts: 1,
                        },
                        LocalResource::Waiting { attempts, .. } if attempts == 5 => {
                            LocalResource::Failed {
                                reason: format!("max attempts reached ({})", attempts),
                            }
                        }
                        LocalResource::Waiting { attempts, .. } => LocalResource::Waiting {
                            reason: "didn't start".to_string(),
                            attempts: attempts + 1,
                        },
                        LocalResource::Ready => LocalResource::Waiting {
                            reason: "was ready, lost?".to_string(),
                            attempts: 0,
                        },
                        LocalResource::Failed { reason: _ } => LocalResource::Failed {
                            reason: "failed again...".to_string(),
                        },
                    };
                    local_states.insert(path_watchable, next);
                }
            }
            let list = list_hash(&hashes);
            let state_value = StateValue::new(list);
            let result = if hashes.len() == jobs_len {
                LocalResult::Ready {
                    observed_status: state_value,
                    local_states,
                }
            } else {
                LocalResult::NotReady {
                    observed_status: state_value,
                    local_states,
                }
            };
            Ok::<_, anyhow::Error>(result)
        };

        Box::pin(future.into_actor(self).map(move |res, act, ctx| {
            let Ok(res) = res else {
                tracing::error!("Failed to get ConnectedWatchers: {:?}", res.unwrap_err());
                return Ok(());
            };

            match res {
                LocalResult::Ready {
                    observed_status: observed_local,
                    local_states,
                } => {
                    act.local_state = LocalState {
                        inner: local_states,
                    };
                    match act.status {
                        ServiceStatus::Idle => {
                            todo!("how?")
                        }
                        ServiceStatus::Reconciling { desired, .. } if desired == observed_local => {
                            tracing::debug!("will set ready, desired matched local");
                            act.status = ServiceStatus::Ready {
                                observed: observed_local,
                            };
                        }
                        ServiceStatus::Reconciling {
                            desired: _,
                            observed: observed_prev,
                        } => {
                            tracing::debug!("still reconciling, but now it's a new thing");
                            act.status = ServiceStatus::Reconciling {
                                desired: observed_local,
                                observed: observed_prev,
                            };
                        }
                        ServiceStatus::Ready { observed } if observed == observed_local => {
                            tracing::debug!("nothing to do!");
                        }
                        ServiceStatus::Ready { observed } => {
                            tracing::debug!("downgrading to reconciling");
                            act.status = ServiceStatus::Reconciling {
                                desired: observed_local,
                                observed: Some(observed),
                            };
                        }
                    }
                }
                LocalResult::NotReady {
                    observed_status: observed_local,
                    local_states,
                } => {
                    tracing::debug!("local result was not ready, setting to reconciling...");
                    act.local_state = LocalState {
                        inner: local_states,
                    };
                    let observed_prev = act.status.observed();
                    act.status = ServiceStatus::Reconciling {
                        desired: observed_local,
                        observed: observed_prev,
                    };
                    // match act.status {}
                }
            }
            state_writer.do_send(MonitorStatusWriter {
                monitor_status: act.status,
                monitor_resource_status: act.local_state.as_resource_status(),
            });
            ctx.notify_later(MonitorAny, Duration::from_secs(1));
            Ok(())
        }))
    }
}

#[derive(actix::Message)]
#[rtype(result = "Result<(), anyhow::Error>")]
struct TryEvict;

impl actix::Handler<TryEvict> for PathMonitors {
    type Result = ResponseActFuture<Self, Result<(), anyhow::Error>>;

    fn handle(&mut self, _msg: TryEvict, _ctx: &mut Self::Context) -> Self::Result {
        let addrs = self.path_monitors.clone();
        let addr_len = addrs.len();

        let future = async move {
            tracing::debug!(
                "will ask {} addresses for their ConnectedWatchers",
                addr_len
            );
            let mut removals = vec![];
            for (path_watchable, addr) in addrs {
                let re = addr.send(ConnectedWatchers).await?;
                if re.observed_len != re.desired_len {
                    removals.push(path_watchable);
                }
            }

            if !removals.is_empty() {
                return Ok::<_, anyhow::Error>(Some(removals));
            }

            Ok::<_, anyhow::Error>(None)
        };

        Box::pin(future.into_actor(self).map(|res, act, _ctx| {
            if let Ok(Some(removals)) = res {
                tracing::debug!(
                    "removing {} path watchables because they never connected",
                    removals.len()
                );
                for path_watchable in removals {
                    act.path_monitors.remove(&path_watchable);
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
