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
    fn handle(&mut self, msg: RequestWatchPath, _ctx: &mut Self::Context) -> Self::Result {
        let Some(watcher) = self.watcher.as_mut() else {
            todo!("Can this ever be reached?");
        };

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
                self.receiver.do_send(FsEvent {
                    kind: evt,
                    fs_event_ctx: self.ctx,
                })
            }
            Err(err) => {
                tracing::error!("cannot watch: {}", err);
                let evt = FsEventKind::PathNotFoundError(PathEvent {
                    path: msg.path.clone(),
                });
                self.receiver.do_send(FsEvent {
                    kind: evt,
                    fs_event_ctx: self.ctx,
                });
                _ctx.stop();
            }
        }
    }
}
