use crate::input_fs::from_input_path;
use crate::task::{TaskCommand, TaskManager};
use crate::tasks::notify_servers::NotifyServers;
use crate::{BsSystem, OverrideInput};
use actix::{Actor, ActorFutureExt, AsyncContext, Handler, ResponseActFuture, WrapFuture};
use bsnext_core::servers_supervisor::file_changed_handler::FileChanged;
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal::{AnyEvent, InternalEvents};
use bsnext_dto::{StoppedWatchingDTO, WatchingDTO};
use bsnext_fs::{
    BufferedChangeEvent, ChangeEvent, FsEvent, FsEventContext, FsEventKind, PathAddedEvent,
    PathEvent,
};
use bsnext_input::{Input, InputError, PathDefinition, PathDefs, PathError};

impl actix::Handler<FsEvent> for BsSystem {
    type Result = ();
    fn handle(&mut self, msg: FsEvent, ctx: &mut Self::Context) -> Self::Result {
        let next = match &msg.kind {
            FsEventKind::ChangeBuffered(buffer_change) => {
                self.handle_buffered(&msg, buffer_change, ctx)
            }
            FsEventKind::Change(inner) => match self.handle_any_change(&msg, inner) {
                // if the change included a new Input, use it
                Some((evt, Some(input))) => {
                    tracing::info!("will override input");
                    ctx.notify(OverrideInput {
                        input,
                        original_event: evt,
                    });
                    // return None here so that the event is not published yet (the updated Input will do it)
                    None
                }
                // otherwise just publish the change as usual
                Some((evt, None)) => Some(evt),
                None => None,
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

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
struct Trigger {
    ctx: FsEventContext,
    task: TaskCommand,
}

impl Trigger {
    fn new(p0: FsEventContext, task: TaskCommand) -> Self {
        Self { ctx: p0, task }
    }
}

impl Handler<Trigger> for BsSystem {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: Trigger, ctx: &mut Self::Context) -> Self::Result {
        let fs_ctx = msg.ctx;
        let fs_ctx_c = fs_ctx.clone();
        let add = self.tasks.get(&fs_ctx);
        if let Some(tm) = add {
            let cloned = tm.clone();
            let addr = cloned.addr().clone();
            let f = async move {
                let r = addr.send(msg.task).await;
                match r {
                    Ok(_) => tracing::trace!("did send"),
                    Err(_) => tracing::trace!("did error"),
                }
                ()
            }
            .into_actor(self)
            .map(move |a, actor, ctx| {
                actor.tasks.remove(&fs_ctx_c);
                tracing::debug!("remaining tasks after removal {}", &actor.tasks.len());
                ()
            });
            Box::pin(f)
        } else {
            Box::pin(async {}.into_actor(self))
        }
    }
}

impl BsSystem {
    fn handle_buffered(
        &mut self,
        msg: &FsEvent,
        buf: &BufferedChangeEvent,
        ctx: &mut <BsSystem as Actor>::Context,
    ) -> Option<AnyEvent> {
        tracing::debug!(msg.event_count = buf.events.len(), msg.ctx = ?msg.ctx, ?buf, "handle_buffered");
        let paths = buf
            .events
            .iter()
            .map(|evt| evt.absolute.to_owned())
            .collect::<Vec<_>>();
        let as_strings = paths
            .iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect::<Vec<String>>();

        ///
        ///
        /// Try to find the matching 'watchable' so that we can decide which tasks to run
        ///
        ///
        let matching_monitor = self
            .any_monitors
            .iter()
            .find(|(path_watchable, any_monitor)| {
                any_monitor.watchable_hash() == msg.ctx.origin_id
            });
        if let Some((a, b)) = matching_monitor {
            println!("Use path_watchable here {:?}", a.spec_opts()?.run);
        }

        ///
        ///
        /// An example of creating a 'NotifyServers' task from the current state of BsSystem
        ///
        ///
        if let Some(servers) = &self.servers_addr {
            self.tasks.entry(msg.ctx.clone()).or_insert_with(|| {
                let d = NotifyServers::new(servers, msg.ctx.clone());
                let addr = d.start().recipient();
                TaskManager::new(addr)
            });
            ctx.notify(Trigger::new(msg.ctx.clone(), TaskCommand::Changes(paths)));
        }

        // todo(alpha): need to exclude changes to the input file if this event has captured it
        Some(AnyEvent::External(ExternalEventsDTO::FilesChanged(
            bsnext_dto::FilesChangedDTO { paths: as_strings },
        )))
    }
    fn handle_any_change(
        &mut self,
        msg: &FsEvent,
        inner: &ChangeEvent,
    ) -> Option<(AnyEvent, Option<Input>)> {
        match msg.ctx.id() {
            0 => self.handle_input_change(inner),
            _ => {
                tracing::trace!(?inner, "Other file changed");
                if let Some(servers) = &self.servers_addr {
                    servers.do_send(FileChanged {
                        path: inner.absolute_path.clone(),
                        ctx: msg.ctx.clone(),
                    })
                }
                Some((
                    AnyEvent::External(ExternalEventsDTO::FileChanged(
                        bsnext_dto::FileChangedDTO::from_path_buf(&inner.path),
                    )),
                    None,
                ))
            }
        }
    }
    fn handle_input_change(&mut self, inner: &ChangeEvent) -> Option<(AnyEvent, Option<Input>)> {
        tracing::info!("InputFile file changed {:?}", inner);

        let ctx = self
            .input_monitors
            .as_ref()
            .map(|x| x.input_ctx.clone())
            .unwrap_or_default();

        let input = from_input_path(&inner.absolute_path, &ctx);

        let Ok(input) = input else {
            let err = input.unwrap_err();
            return Some((AnyEvent::Internal(InternalEvents::InputError(*err)), None));
        };

        Some((
            AnyEvent::External(ExternalEventsDTO::InputFileChanged(
                bsnext_dto::FileChangedDTO::from_path_buf(&inner.path),
            )),
            Some(input),
        ))
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
}
