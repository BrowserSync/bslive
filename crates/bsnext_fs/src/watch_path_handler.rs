use crate::actor::FsWatcher;
use crate::{FsEvent, FsEventKind, PathAddedEvent, PathEvent};
use actix::{ActorContext, Handler};
use notify::{RecursiveMode, Watcher};
use std::path::PathBuf;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct RequestWatchPath {
    pub path: PathBuf,
}

impl Handler<RequestWatchPath> for FsWatcher {
    type Result = ();

    // todo: ensure this isn't sent for every input change
    fn handle(&mut self, msg: RequestWatchPath, ctx: &mut Self::Context) -> Self::Result {
        let Some(watcher) = self.watcher.as_mut() else {
            todo!("Can this ever be reached?");
        };

        tracing::debug!(?self.cwd, ?msg.path, "will watch msg.path");
        match watcher.watch(&msg.path, RecursiveMode::Recursive) {
            Ok(_) => {
                // tracing::debug!(path = ?msg.path, "👀 watching! {} receivers", self.receivers.len());
                tracing::debug!(?self.cwd);

                let matched = msg.path == self.cwd;

                let relative = if matched {
                    msg.path.clone()
                } else {
                    match msg.path.strip_prefix(&self.cwd) {
                        Ok(stripped) => stripped.to_path_buf(),
                        Err(e) => {
                            tracing::trace!(?e, "could not extract the CWD from a path");
                            msg.path.clone()
                        }
                    }
                };
                let evt = FsEventKind::PathAdded(PathAddedEvent {
                    path: relative.clone(),
                });
                self.receiver.do_send(FsEvent::new(evt, self.ctx))
            }
            Err(err) => {
                tracing::debug!("cannot watch, error: {}", err);
                let evt = FsEventKind::PathNotFoundError(PathEvent {
                    path: msg.path.clone(),
                });
                self.receiver.do_send(FsEvent::new(evt, self.ctx));
                ctx.stop();
            }
        }
    }
}
