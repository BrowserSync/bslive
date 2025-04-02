use crate::input_fs::from_input_path;
use crate::runner::{Runnable, Runner};
use crate::task::{AsActor, Task, TaskCommand, TaskComms, TaskGroup};
use crate::{BsSystem, OverrideInput};
use actix::{ActorFutureExt, AsyncContext, Handler, ResponseActFuture, WrapFuture};
use bsnext_core::servers_supervisor::file_changed_handler::FileChanged;
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal::{AnyEvent, InternalEvents};
use bsnext_dto::{StoppedWatchingDTO, WatchingDTO};
use bsnext_fs::{
    BufferedChangeEvent, ChangeEvent, FsEvent, FsEventContext, FsEventKind, PathAddedEvent,
    PathEvent,
};
use bsnext_input::route::BsLiveRunner;
use bsnext_input::{Input, InputError, PathDefinition, PathDefs, PathError};

impl actix::Handler<FsEvent> for BsSystem {
    type Result = ();
    fn handle(&mut self, msg: FsEvent, ctx: &mut Self::Context) -> Self::Result {
        let next = match &msg.kind {
            FsEventKind::ChangeBuffered(buffer_change) => {
                let (task, cmd) = self.handle_buffered(&msg, buffer_change);
                ctx.notify(Trigger::new(task, cmd));
                None
            }
            FsEventKind::Change(inner) => match self.handle_any_change(&msg, inner) {
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
                // otherwise just publish the change as usual
                (evt, None) => Some(evt),
            },
            FsEventKind::PathAdded(path) => self.handle_path_added(path),
            FsEventKind::PathRemoved(path) => self.handle_path_removed(path),
            FsEventKind::PathNotFoundError(pdo) => self.handle_path_not_found(pdo),
        };
        if let Some(any_event) = next {
            tracing::debug!("will publish any_event {:?}", any_event);
            self.publish_any_event(any_event)
        }
    }
}

#[derive(actix::Message, Debug)]
#[rtype(result = "()")]
struct Trigger {
    task: Task,
    cmd: TaskCommand,
}

impl Trigger {
    fn new(task: Task, cmd: TaskCommand) -> Self {
        Self { task, cmd }
    }

    pub fn cmd(&self) -> TaskCommand {
        self.cmd.clone()
    }

    pub fn fs_ctx(&self) -> &FsEventContext {
        match &self.cmd {
            TaskCommand::Changes {
                fs_event_context, ..
            } => fs_event_context,
        }
    }
}

impl Handler<Trigger> for BsSystem {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: Trigger, _ctx: &mut Self::Context) -> Self::Result {
        let cmd = msg.cmd();
        let fs_ctx = msg.fs_ctx();
        let entry = self.tasks.get(fs_ctx);
        let cloned_id = fs_ctx.clone();

        if let Some(entry) = entry {
            tracing::info!("ignoring concurrent task triggering: prev: {}", entry);
            return Box::pin(async {}.into_actor(self));
        }

        self.tasks.insert(fs_ctx.clone(), 0);
        let cmd_recip = Box::new(msg.task).into_actor2();

        Box::pin(
            cmd_recip
                .send(cmd)
                .into_actor(self)
                .map(move |_resp, actor, _| {
                    actor.tasks.remove(&cloned_id);
                }),
        )
    }
}

impl BsSystem {
    fn handle_buffered(&mut self, msg: &FsEvent, buf: &BufferedChangeEvent) -> (Task, TaskCommand) {
        tracing::debug!(msg.event_count = buf.events.len(), msg.ctx = ?msg.fs_event_ctx, ?buf, "handle_buffered");
        let paths = buf
            .events
            .iter()
            .map(|evt| evt.absolute.to_owned())
            .collect::<Vec<_>>();

        let runner = self.as_runner(&msg.fs_event_ctx);

        let (Some(any_event_sender), Some(servers_addr)) =
            (&self.any_event_sender, &self.servers_addr)
        else {
            todo!("must have these senders...?");
        };

        let cmd = TaskCommand::Changes {
            changes: paths,
            fs_event_context: msg.fs_event_ctx.clone(),
            task_comms: TaskComms::new(
                any_event_sender.clone(),
                Some(servers_addr.clone().recipient()),
            ),
            invocation_id: 0,
        };

        let task_group = TaskGroup::from(runner);

        (Task::Group(task_group), cmd)
    }

    fn handle_any_change(
        &mut self,
        msg: &FsEvent,
        inner: &ChangeEvent,
    ) -> (AnyEvent, Option<Input>) {
        match msg.fs_event_ctx.id() {
            0 => self.handle_input_change(inner),
            _ => {
                tracing::trace!(?inner, "Other file changed");
                if let Some(servers) = &self.servers_addr {
                    servers.do_send(FileChanged {
                        path: inner.absolute_path.clone(),
                        ctx: msg.fs_event_ctx.clone(),
                    })
                }
                (
                    AnyEvent::External(ExternalEventsDTO::FileChanged(
                        bsnext_dto::FileChangedDTO::from_path_buf(&inner.path),
                    )),
                    None,
                )
            }
        }
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

    fn as_runner(&self, fs_event_ctx: &FsEventContext) -> Runner {
        tracing::info!("as_runner {:?}", fs_event_ctx);
        let matching_monitor = self
            .any_monitors
            .iter()
            .find(|(_, any_monitor)| any_monitor.watchable_hash() == fs_event_ctx.origin_id);

        if let Some((path_watchable, _any_monitor)) = matching_monitor {
            tracing::info!("path_watchable {:?}", path_watchable);
            let default_task = Runner::seq(&[
                Runnable::BsLive(BsLiveRunner::NotifyServer),
                Runnable::BsLive(BsLiveRunner::ExtEvent),
            ]);
            let run = path_watchable
                .runner()
                .map(ToOwned::to_owned)
                .unwrap_or(default_task);

            return run;
        }

        Runner::seq(&[])
    }
}
