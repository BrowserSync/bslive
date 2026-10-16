use crate::fs_task_tracker::TriggerFsTask;
use crate::monitor_input::InputMonitor;
use crate::system::{BsSystem, ResolveInputAgain};
use crate::tasks::task_spec::TaskSpec;
use crate::tasks::Runnable;
use actix::{Addr, AsyncContext};
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_core::servers_supervisor::file_changed_handler::FileChanged;
use bsnext_dto::any_event::AnyEvent;
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::{InputErrorDetailDTO, StoppedWatchingDTO, WatchingDTO};
use bsnext_fs::{
    BufferedChangeset, Debounce, FsEvent, FsEventContext, FsEventKind, PathAddedEvent,
    PathDescriptionOwned, PathEvent,
};
use bsnext_input::bs_live_built_in_task::BsLiveBuiltInTask;
use bsnext_input::route::WatchSpec;
use bsnext_path_monitor::{Changeset, PathMonitorChangeset};
use bsnext_task::task_trigger::FsChangesTrigger;
use std::path::PathBuf;
use tracing::{debug, debug_span, info};

impl actix::Handler<PathMonitorChangeset> for BsSystem {
    type Result = ();

    fn handle(&mut self, msg: PathMonitorChangeset, ctx: &mut Self::Context) -> Self::Result {
        let addr = ctx.address();
        let span = debug_span!("Handler->FsEventGrouping->BsSystem");
        let _guard = span.enter();
        let debounce = msg.debounce;
        let watch_spec = msg.watch_spec;
        let next = match msg.changeset {
            Changeset::Singular(fs_event) => {
                tracing::debug!("will handle single event");
                if let Some(evt) = self.handle_fs_event(fs_event, addr, debounce) {
                    self.publish_external_event(AnyEvent::External(evt));
                }
                None
            }
            Changeset::BufferedChange(buff) => {
                if let Some((task_trigger, task_spec)) =
                    handle_buffered(self.input_monitors.as_ref(), buff, watch_spec)
                {
                    tracing::debug!("will trigger task runner");
                    self.fs_task_tracker
                        .do_send(TriggerFsTask::new(task_spec, task_trigger));
                } else {
                    tracing::debug!("will NOT trigger task runner");
                }
                None
            }
        };
        if let Some(any_event) = next {
            tracing::trace!("will publish any_event {:?}", any_event);
            self.publish_external_event(any_event)
        }
    }
}

impl BsSystem {
    fn handle_fs_event(
        &mut self,
        fs_event: FsEvent,
        addr: Addr<Self>,
        debounce: Debounce,
    ) -> Option<ExternalEventsDTO> {
        match &fs_event.kind() {
            FsEventKind::Change(ch) if fs_event.fs_event_ctx().is_root() => {
                tracing::info!("fs_event_ctx=root");
                addr.do_send(ResolveInputAgain);
                None
            }
            FsEventKind::Change(inner) => {
                let servers_addr = self.servers().clone();
                let evt = handle_any_change(servers_addr, fs_event.fs_event_ctx(), inner);
                Some(evt)
            }
            FsEventKind::PathAdded(path) => handle_path_added(path, &debounce),
            FsEventKind::PathRemoved(path) => handle_path_removed(path),
            FsEventKind::PathNotFoundError(pdo) => handle_path_not_found(self.cwd.clone(), pdo),
        }
    }
}

#[tracing::instrument(skip_all)]
fn handle_buffered(
    input_monitor: Option<&InputMonitor>,
    buf: BufferedChangeset,
    watch_spec: WatchSpec,
) -> Option<(FsChangesTrigger, TaskSpec)> {
    tracing::debug!(msg.event_count = buf.changes.len(), msg.ctx = ?buf.fs_event_ctx, ?buf);

    let change = if let Some(mon) = input_monitor {
        if let Some(fp) = mon.input_ctx.file_path() {
            tracing::debug!("Dropping input crossover {}", fp.display());
            buf.dropping_absolute(fp)
        } else {
            buf
        }
    } else {
        buf
    };

    if change.changes.is_empty() {
        tracing::debug!(
            "Ignoring handle_buffered events because it was empty after removing input monitor"
        );
        return None;
    }

    let paths = change
        .changes
        .iter()
        .map(|evt| evt.absolute.to_owned())
        .collect::<Vec<_>>();

    let task_spec = task_spec_for_fs_event(&watch_spec);
    let trigger = FsChangesTrigger::new(paths, change.fs_event_ctx);

    Some((trigger, task_spec))
}

fn handle_any_change(
    servers_addr: Addr<ServersSupervisor>,
    fs_event_ctx: &FsEventContext,
    inner: &PathDescriptionOwned,
) -> ExternalEventsDTO {
    tracing::trace!(?inner, "Other file changed");
    servers_addr.do_send(FileChanged {
        path: inner.absolute.clone(),
        ctx: *fs_event_ctx,
    });
    ExternalEventsDTO::FileChanged(bsnext_dto::FileChangedDTO::from_path_buf(
        inner.relative.as_ref().unwrap_or(&inner.absolute),
    ))
}

fn handle_path_not_found(_cwd: PathBuf, pdo: &PathEvent) -> Option<ExternalEventsDTO> {
    let as_str = pdo.path.to_string_lossy().to_string();
    // let cwd = cwd.clone();
    // let abs = cwd.join(&as_str);
    // let def = PathDefinition {
    //     input: as_str,
    //     cwd: cwd.clone(),
    //     absolute: abs,
    // };
    // let err = InputError::PathError(PathError::MissingPaths {
    //     paths: PathDefs(vec![def]),
    // });

    Some(ExternalEventsDTO::InputError(InputErrorDetailDTO {
        error: format!("{} not found, trying again in 2 second intervals", as_str),
    }))
}

fn handle_path_added(path: &PathAddedEvent, debounce: &Debounce) -> Option<ExternalEventsDTO> {
    Some(ExternalEventsDTO::Watching(WatchingDTO::from_path_buf(
        &path.path, *debounce,
    )))
}

fn handle_path_removed(path: &PathEvent) -> Option<ExternalEventsDTO> {
    Some(ExternalEventsDTO::WatchingStopped(
        StoppedWatchingDTO::from_path_buf(&path.path),
    ))
}

#[tracing::instrument(skip_all)]
fn task_spec_for_fs_event(watch_spec: &WatchSpec) -> TaskSpec {
    if let Some(spec) = TaskSpec::opt_from(watch_spec) {
        info!("matching task_spec: {:?}", spec);
        info!("matching fs_event_ctx: {:?}", watch_spec);
        spec.to_owned()
    } else {
        debug!("creating a default task spec on the fly because fs_event_ctx didn't match any task specs");
        TaskSpec::seq(&[
            Runnable::BsLiveTask(BsLiveBuiltInTask::NotifyServer),
            Runnable::BsLiveTask(BsLiveBuiltInTask::PublishExternalEvent),
        ])
    }
}
