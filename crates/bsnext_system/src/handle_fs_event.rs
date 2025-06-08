use crate::input_fs::from_input_path;
use crate::task::{Task, TaskCommand, TaskComms};
use crate::task_group::TaskGroup;
use crate::task_list::{BsLiveTask, Runnable, TaskList};
use crate::trigger_fs_task::TriggerFsTask;
use crate::{BsSystem, OverrideInput};
use actix::AsyncContext;
use bsnext_core::servers_supervisor::file_changed_handler::FileChanged;
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal::{AnyEvent, InternalEvents};
use bsnext_dto::{StoppedWatchingDTO, WatchingDTO};
use bsnext_fs::{
    BufferedChangeEvent, ChangeEvent, FsEvent, FsEventContext, FsEventKind, PathAddedEvent,
    PathEvent,
};
use bsnext_input::{Input, InputError, PathDefinition, PathDefs, PathError};
use tracing::info;

impl actix::Handler<FsEvent> for BsSystem {
    type Result = ();
    fn handle(&mut self, msg: FsEvent, ctx: &mut Self::Context) -> Self::Result {
        let next = match msg.kind {
            FsEventKind::ChangeBuffered(buffer_change) => {
                if let Some((task, cmd, runner)) =
                    self.handle_buffered(&msg.fs_event_ctx, buffer_change)
                {
                    tracing::debug!("will trigger task runner");
                    ctx.notify(TriggerFsTask::new(task, cmd, runner));
                }
                None
            }
            FsEventKind::Change(inner) if msg.fs_event_ctx.is_root() => {
                match self.handle_input_change(&inner) {
                    // if the change included a new Input, use it
                    (any_event, Some(input)) => {
                        tracing::info!("will override input");
                        ctx.notify(OverrideInput {
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
                let evt = self.handle_any_change(&msg.fs_event_ctx, &inner);
                Some(evt)
            }
            FsEventKind::PathAdded(path) => self.handle_path_added(&path),
            FsEventKind::PathRemoved(path) => self.handle_path_removed(&path),
            FsEventKind::PathNotFoundError(pdo) => self.handle_path_not_found(&pdo),
        };
        if let Some(any_event) = next {
            tracing::trace!("will publish any_event {:?}", any_event);
            self.publish_any_event(any_event)
        }
    }
}

impl BsSystem {
    #[tracing::instrument(skip_all)]
    fn handle_buffered(
        &mut self,
        fs_event_ctx: &FsEventContext,
        buf: BufferedChangeEvent,
    ) -> Option<(Task, TaskCommand, TaskList)> {
        tracing::debug!(msg.event_count = buf.events.len(), msg.ctx = ?fs_event_ctx, ?buf);

        let change = if let Some(mon) = &self.input_monitors {
            if let Some(fp) = mon.input_ctx.file_path() {
                tracing::debug!("Dropping input crossover");
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

        let fs_event_runner = self.as_runner_for_fs_event(fs_event_ctx);

        let cmd = TaskCommand::Changes {
            changes: paths,
            fs_event_context: fs_event_ctx.clone(),
            task_comms: self.task_comms(),
            invocation_id: 0,
        };

        // todo: use this example as a way to display a dry-run scenario
        // let tree = runner.as_tree();
        // let as_str = archy(&tree, None);
        // println!("upcoming-->");
        // println!("{as_str}");
        let task_group = TaskGroup::from(fs_event_runner.clone());

        Some((Task::Group(task_group), cmd, fs_event_runner))
    }

    pub fn task_comms(&mut self) -> TaskComms {
        let (Some(any_event_sender), Some(servers_addr)) =
            (&self.any_event_sender, &self.servers_addr)
        else {
            todo!("must have these senders...?");
        };
        TaskComms::new(
            any_event_sender.clone(),
            Some(servers_addr.clone().recipient()),
        )
    }

    fn handle_any_change(
        &mut self,
        fs_event_ctx: &FsEventContext,
        inner: &ChangeEvent,
    ) -> AnyEvent {
        tracing::trace!(?inner, "Other file changed");
        if let Some(servers) = &self.servers_addr {
            servers.do_send(FileChanged {
                path: inner.absolute_path.clone(),
                ctx: fs_event_ctx.clone(),
            })
        }
        AnyEvent::External(ExternalEventsDTO::FileChanged(
            bsnext_dto::FileChangedDTO::from_path_buf(&inner.path),
        ))
    }
    fn handle_input_change(&mut self, inner: &ChangeEvent) -> (AnyEvent, Option<Input>) {
        tracing::info!("InputFile file changed {:?}", inner);

        let ctx = self
            .input_monitors
            .as_ref()
            .map(|x| x.input_ctx.clone())
            .unwrap_or_default();

        let input = from_input_path(&inner.absolute_path, &ctx);

        let Ok(input) = input else {
            let err = input.unwrap_err();
            return (AnyEvent::Internal(InternalEvents::InputError(*err)), None);
        };

        (
            AnyEvent::External(ExternalEventsDTO::InputFileChanged(
                bsnext_dto::FileChangedDTO::from_path_buf(&inner.path),
            )),
            Some(input),
        )
    }
    fn handle_path_added(&mut self, path: &PathAddedEvent) -> Option<AnyEvent> {
        Some(AnyEvent::External(ExternalEventsDTO::Watching(
            WatchingDTO::from_path_buf(&path.path, path.debounce),
        )))
    }

    fn handle_path_removed(&mut self, path: &PathEvent) -> Option<AnyEvent> {
        Some(AnyEvent::External(ExternalEventsDTO::WatchingStopped(
            StoppedWatchingDTO::from_path_buf(&path.path),
        )))
    }

    fn handle_path_not_found(&mut self, pdo: &PathEvent) -> Option<AnyEvent> {
        let as_str = pdo.path.to_string_lossy().to_string();
        let cwd = self.cwd.clone().unwrap();
        let abs = cwd.join(&as_str);
        let def = PathDefinition {
            input: as_str,
            cwd: self.cwd.clone().unwrap(),
            absolute: abs,
        };
        let e = InputError::PathError(PathError::MissingPaths {
            paths: PathDefs(vec![def]),
        });
        Some(AnyEvent::Internal(InternalEvents::InputError(e)))
    }

    #[tracing::instrument(skip_all)]
    fn as_runner_for_fs_event(&self, fs_event_ctx: &FsEventContext) -> TaskList {
        let matching_monitor = self
            .any_monitors
            .iter()
            .find(|(_, path_monitor)| path_monitor.fs_ctx == (*fs_event_ctx));

        if let Some((path_watchable, _any_monitor)) = matching_monitor {
            info!("matching monitor, path_watchable: {}", path_watchable);
            info!("matching fs_event_ctx: {:?}", fs_event_ctx);
            let custom_task_list = path_watchable.task_list();
            if let None = custom_task_list {
                info!("no custom tasks given, NotifyServer + ExtEvent will be defaults");
            }
            let runner = custom_task_list.map(ToOwned::to_owned).unwrap_or_else(|| {
                TaskList::seq(&[
                    Runnable::BsLiveTask(BsLiveTask::NotifyServer),
                    Runnable::BsLiveTask(BsLiveTask::ExtEvent),
                ])
            });

            return runner;
        }

        TaskList::seq(&[])
    }
}
