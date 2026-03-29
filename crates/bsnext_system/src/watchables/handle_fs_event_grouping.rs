use crate::fs_task_tracker::TriggerFsTask;
use crate::input_fs::from_input_path;
use crate::override_input::OverrideInput;
use crate::system::BsSystem;
use crate::tasks::bs_live_task::BsLiveTask;
use crate::tasks::task_comms::TaskComms;
use crate::tasks::task_spec::TaskSpec;
use crate::tasks::Runnable;
use crate::watchables::path_monitor::PathMonitorMeta;
use crate::watchables::path_watchable::PathWatchable;
use actix::{Addr, AsyncContext};
use bsnext_core::servers_supervisor::file_changed_handler::FileChanged;
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal::{AnyEvent, InternalEvents};
use bsnext_dto::{StoppedWatchingDTO, WatchingDTO};
use bsnext_fs::{
    BufferedChangeEvent, FsEvent, FsEventContext, FsEventGrouping, FsEventKind, PathAddedEvent,
    PathDescriptionOwned, PathEvent,
};
use bsnext_input::{Input, InputError, PathDefinition, PathDefs, PathError};
use bsnext_task::task_trigger::FsChangesTrigger;
use tracing::{debug_span, info};

impl actix::Handler<FsEventGrouping> for BsSystem {
    type Result = ();

    fn handle(&mut self, msg: FsEventGrouping, ctx: &mut Self::Context) -> Self::Result {
        let addr = ctx.address();
        let span = debug_span!("Handler->FsEventGrouping->BsSystem");
        let _guard = span.enter();
        let next = match msg {
            FsEventGrouping::Singular(fs_event) => self.handle_fs_event(fs_event, addr),
            FsEventGrouping::BufferedChange(buff) => {
                if let Some((task_trigger, task_spec)) = self.handle_buffered(buff) {
                    tracing::debug!("will trigger task runner");
                    self.fs_task_tracker
                        .do_send(TriggerFsTask::new(task_spec, task_trigger));
                }
                None
            }
        };
        if let Some(any_event) = next {
            tracing::trace!("will publish any_event {:?}", any_event);
            self.publish_any_event(any_event)
        }
    }
}

impl BsSystem {
    fn handle_fs_event(&mut self, fs_event: FsEvent, addr: Addr<Self>) -> Option<AnyEvent> {
        match &fs_event.kind {
            FsEventKind::Change(ch) if fs_event.fs_event_ctx.is_root() => {
                tracing::info!("fs_event_ctx=root");
                match self.handle_input_change(ch) {
                    // if the change included a new Input, use it
                    (any_event, Some(input)) => {
                        tracing::info!("will override input");
                        addr.do_send(OverrideInput {
                            input,
                            original_event: any_event,
                        });
                        // return None here so that the event is not published yet (the updated Input will do it)
                        None
                    }
                    // otherwise publish the change as usual
                    (evt, None) => Some(evt),
                }
            }
            FsEventKind::Change(inner) => {
                let evt = self.handle_any_change(&fs_event.fs_event_ctx, inner);
                Some(evt)
            }
            FsEventKind::PathAdded(path) => {
                let Some(pw) = self.monitor_meta(&fs_event.fs_event_ctx) else {
                    tracing::error!(evt=?fs_event, "missing monitor meta data");
                    return None;
                };
                self.handle_path_added(path, pw)
            }
            FsEventKind::PathRemoved(path) => self.handle_path_removed(path),
            FsEventKind::PathNotFoundError(pdo) => self.handle_path_not_found(pdo),
        }
    }
    fn monitor_meta(&self, incoming: &FsEventContext) -> Option<&PathMonitorMeta> {
        if incoming.is_root() {
            self.input_monitors.as_ref().map(|m| &m.monitor_meta)
        } else {
            self.any_monitors
                .iter()
                .find(|(.., (_addr, PathMonitorMeta { ref fs_ctx, .. }))| fs_ctx == incoming)
                .map(|(.., (_addr, meta))| meta)
        }
    }
    fn path_watchable(&self, incoming: &FsEventContext) -> Option<&PathWatchable> {
        self.any_monitors
            .iter()
            .find(|(.., (_path_monitor, PathMonitorMeta { ref fs_ctx, .. }))| fs_ctx == incoming)
            .map(|(pw, ..)| pw)
    }
    #[tracing::instrument(skip_all)]
    fn handle_buffered(
        &mut self,
        buf: BufferedChangeEvent,
    ) -> Option<(FsChangesTrigger, TaskSpec)> {
        tracing::debug!(msg.event_count = buf.events.len(), msg.ctx = ?buf.fs_ctx, ?buf);

        let change = if let Some(mon) = &self.input_monitors {
            if let Some(fp) = mon.input_ctx.file_path() {
                tracing::debug!("Dropping input crossover {}", fp.display());
                buf.dropping_absolute(fp)
            } else {
                buf
            }
        } else {
            buf
        };

        if change.events.is_empty() {
            tracing::debug!(
                "Ignoring handle_buffered events because it was empty after removing input monitor"
            );
            return None;
        }

        let paths = change
            .events
            .iter()
            .map(|evt| evt.absolute.to_owned())
            .collect::<Vec<_>>();

        let task_spec = self.task_spec_for_fs_event(&change.fs_ctx);
        let trigger = FsChangesTrigger::new(paths, change.fs_ctx);

        Some((trigger, task_spec))
    }

    pub fn task_comms(&mut self) -> TaskComms {
        TaskComms::new(self.sender().clone())
    }

    fn handle_any_change(
        &mut self,
        fs_event_ctx: &FsEventContext,
        inner: &PathDescriptionOwned,
    ) -> AnyEvent {
        tracing::trace!(?inner, "Other file changed");
        self.servers().do_send(FileChanged {
            path: inner.absolute.clone(),
            ctx: *fs_event_ctx,
        });
        AnyEvent::External(ExternalEventsDTO::FileChanged(
            bsnext_dto::FileChangedDTO::from_path_buf(
                inner.relative.as_ref().unwrap_or(&inner.absolute),
            ),
        ))
    }
    fn handle_input_change(&mut self, inner: &PathDescriptionOwned) -> (AnyEvent, Option<Input>) {
        tracing::info!("InputFile file changed {:?}", inner);

        let ctx = self
            .input_monitors
            .as_ref()
            .map(|x| x.input_ctx.clone())
            .unwrap_or_default();

        let input = from_input_path(&inner.absolute, &ctx);

        let Ok(input) = input else {
            let err = input.unwrap_err();
            return (AnyEvent::Internal(InternalEvents::InputError(*err)), None);
        };
        let path_to_report = inner.relative.as_ref().unwrap_or(&inner.absolute);

        (
            AnyEvent::External(ExternalEventsDTO::InputFileChanged(
                bsnext_dto::FileChangedDTO::from_path_buf(path_to_report),
            )),
            Some(input),
        )
    }
    fn handle_path_added(&self, path: &PathAddedEvent, meta: &PathMonitorMeta) -> Option<AnyEvent> {
        Some(AnyEvent::External(ExternalEventsDTO::Watching(
            WatchingDTO::from_path_buf(&path.path, meta.debounce),
        )))
    }

    fn handle_path_removed(&mut self, path: &PathEvent) -> Option<AnyEvent> {
        Some(AnyEvent::External(ExternalEventsDTO::WatchingStopped(
            StoppedWatchingDTO::from_path_buf(&path.path),
        )))
    }

    fn handle_path_not_found(&mut self, pdo: &PathEvent) -> Option<AnyEvent> {
        let as_str = pdo.path.to_string_lossy().to_string();
        let cwd = self.cwd.clone();
        let abs = cwd.join(&as_str);
        let def = PathDefinition {
            input: as_str,
            cwd: self.cwd.clone(),
            absolute: abs,
        };
        let e = InputError::PathError(PathError::MissingPaths {
            paths: PathDefs(vec![def]),
        });
        Some(AnyEvent::Internal(InternalEvents::InputError(e)))
    }

    #[tracing::instrument(skip_all)]
    fn task_spec_for_fs_event(&self, fs_event_ctx: &FsEventContext) -> TaskSpec {
        let Some(path_watchable) = self.path_watchable(fs_event_ctx) else {
            tracing::error!("did not find a matching monitor");
            return TaskSpec::seq(&[]);
        };

        info!("matching monitor, path_watchable: {}", path_watchable);
        info!("matching fs_event_ctx: {:?}", fs_event_ctx);

        let custom_task_spec = path_watchable.task_spec();
        if custom_task_spec.is_none() {
            info!("no custom tasks given, NotifyServer + ExtEvent will be defaults");
        }
        custom_task_spec.map(ToOwned::to_owned).unwrap_or_else(|| {
            TaskSpec::seq(&[
                Runnable::BsLiveTask(BsLiveTask::NotifyServer),
                Runnable::BsLiveTask(BsLiveTask::PublishExternalEvent),
            ])
        })
    }
}
