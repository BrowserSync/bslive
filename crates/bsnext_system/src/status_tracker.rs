use crate::system::Accept;
use actix::{Actor, Handler, ResponseFuture};
use bsnext_dto::any_event::AnyEvent;
use bsnext_dto::status_events::{
    monitor_status_hash, server_status_hash, MonitorStatusReader, MonitorStatusWriter,
    ServerStatusReader, ServerStatusWriter, ServiceStatus, StateValue, Transition,
};
use bsnext_dto::status_events_dto::ResourceStatus;
use bsnext_input::watchables::path_watchable::PathWatchable;
use bsnext_input::watchables::PathWatchables;
use bsnext_input::Input;
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use tracing::debug_span;

pub struct StatusTracker {
    input: Input,
    cwd: PathBuf,
    servers_status: ServiceStatus,
    monitor_status: ServiceStatus,
    monitor_resource_status: HashMap<PathWatchable, ResourceStatus>,
    sender: Sender<AnyEvent>,
}

impl Actor for StatusTracker {
    type Context = actix::Context<Self>;
}

impl Handler<Accept> for StatusTracker {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: Accept, _ctx: &mut Self::Context) -> Self::Result {
        self.mark_servers(&msg.input);
        self.mark_monitors(&msg.input);
        self.input = msg.input;
        self.cwd = msg.cwd;
        tracing::info!(status = ?self.servers_status, "did mark servers, next state");
        tracing::info!(status = ?self.monitor_status, "did mark monitors, next state");
        Box::pin(async move {
            let _a = "";
        })
    }
}

impl StatusTracker {
    pub fn new(cwd: PathBuf, sender: Sender<AnyEvent>) -> Self {
        Self {
            servers_status: ServiceStatus::default(),
            monitor_status: ServiceStatus::default(),
            monitor_resource_status: Default::default(),
            input: Input::default(),
            cwd,
            sender,
        }
    }
    fn mark_servers(&mut self, input: &Input) {
        tracing::info!("will mark servers");
        let id = server_status_hash(&input.servers);
        let desired = StateValue::new(id);
        self.servers_status = match self.servers_status {
            ServiceStatus::Idle => ServiceStatus::Reconciling {
                observed: None,
                desired,
            },
            ServiceStatus::Reconciling { observed, .. } => {
                ServiceStatus::Reconciling { observed, desired }
            }
            ServiceStatus::Ready { observed } if observed != desired => {
                ServiceStatus::Reconciling {
                    observed: Some(observed),
                    desired,
                }
            }
            ServiceStatus::Ready { observed } => ServiceStatus::Ready { observed },
        };
        tracing::info!(id = id, "hasher for servers")
    }
    fn mark_monitors(&mut self, input: &Input) {
        tracing::info!("will mark monitors");
        let path_watchables = PathWatchables::new(input);
        let id = monitor_status_hash(&path_watchables);
        let next_desired = StateValue::new(id);
        self.monitor_status = match self.monitor_status {
            ServiceStatus::Idle => ServiceStatus::Reconciling {
                observed: None,
                desired: next_desired,
            },
            ServiceStatus::Reconciling {
                // here we drop the old desired state
                desired: _,
                observed,
            } => ServiceStatus::Reconciling {
                observed,
                desired: next_desired,
            },
            ServiceStatus::Ready { observed } if observed != next_desired => {
                ServiceStatus::Reconciling {
                    observed: Some(observed),
                    desired: next_desired,
                }
            }
            ServiceStatus::Ready { observed } => ServiceStatus::Ready { observed },
        };
        tracing::info!(id = id, "hasher for servers");
        // let id = server_status_hash(&input.servers);
        // let desired = StateValue::new(id);
        // todo!("how to compute monitors?")
    }
}

impl Handler<ServerStatusReader> for StatusTracker {
    type Result = ResponseFuture<(ServiceStatus, Input)>;

    fn handle(&mut self, _msg: ServerStatusReader, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(futures::future::ready((
            self.servers_status,
            self.input.clone(),
        )))
    }
}

impl Handler<MonitorStatusReader> for StatusTracker {
    type Result = ResponseFuture<(ServiceStatus, Input)>;

    fn handle(&mut self, _msg: MonitorStatusReader, _ctx: &mut Self::Context) -> Self::Result {
        Box::pin(futures::future::ready((
            self.monitor_status,
            self.input.clone(),
        )))
    }
}

impl Handler<MonitorStatusWriter> for StatusTracker {
    type Result = ();

    fn handle(&mut self, msg: MonitorStatusWriter, _ctx: &mut Self::Context) -> Self::Result {
        let span = tracing::info_span!("monitor status");
        let _g = span.enter();
        tracing::debug!(prev.status = ?self.monitor_status, " --> ");
        tracing::debug!(prev.resource_status = ?self.monitor_resource_status, " --> ");
        tracing::debug!(next.status = ?msg.monitor_status, " --> ");
        tracing::debug!(next.resource_status = ?msg.monitor_resource_status, " --> ");
        for (id, status) in &msg.monitor_resource_status {
            let span = debug_span!("monitor resource status", id = ?id);
            let _g = span.enter();
            let did_transition = self
                .monitor_resource_status
                .get(id)
                .is_some_and(|x| x != status);
            tracing::info!(did_transition);
            if did_transition {
                let _evt = Transition::path_monitor(status.clone(), id.clone());
                todo!("publish the event!");
                // dbg!(&evt);
                // self.sender.send(evt).await.unwrap();
                match status {
                    ResourceStatus::Pending => {}
                    ResourceStatus::Waiting { reason } => {
                        tracing::info!("waiting! reason: {reason}")
                    }
                    ResourceStatus::Ready => tracing::info!("ready!"),
                    ResourceStatus::Failed { reason } => tracing::info!("failed reason: {reason}!"),
                }
            }
        }
        self.monitor_status = msg.monitor_status;
        self.monitor_resource_status = msg.monitor_resource_status;
    }
}

impl Handler<ServerStatusWriter> for StatusTracker {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: ServerStatusWriter, _ctx: &mut Self::Context) -> Self::Result {
        tracing::info!("status writer for servers");
        tracing::info!(next.status = ?msg.servers_status, " --> ");
        tracing::info!(prev.status = ?self.servers_status, " --> ");
        // dbg!(&msg.server_status);
        self.servers_status = msg.servers_status;
        Box::pin(async move {})
    }
}
