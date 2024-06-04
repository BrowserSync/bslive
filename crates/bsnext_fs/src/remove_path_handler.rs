use crate::actor::FsWatcher;
use crate::{FsEvent, FsEventKind, FsWatchError, PathEvent};
use actix::Handler;
use notify::Watcher;
use std::path::PathBuf;

#[derive(actix::Message)]
#[rtype(result = "Result<(), FsWatchError>")]
pub struct RemoveWatchPath {
    pub path: PathBuf,
}

impl Handler<RemoveWatchPath> for FsWatcher {
    type Result = Result<(), FsWatchError>;

    fn handle(&mut self, msg: RemoveWatchPath, _ctx: &mut Self::Context) -> Self::Result {
        tracing::trace!(path = ?msg.path, "-> RemoveWatchPath");
        if let Some(watcher) = self.watcher.as_mut() {
            match watcher.unwatch(&msg.path) {
                Ok(_) => {
                    tracing::info!(?msg.path, "removed!");
                    let relative = match msg.path.strip_prefix(&self.cwd) {
                        Ok(stripped) => stripped.to_path_buf(),
                        Err(e) => {
                            tracing::debug!(?e, "could not extract the CWD from a path");
                            msg.path.clone()
                        }
                    };
                    for recip in &self.receivers {
                        let evt = FsEventKind::PathRemoved(PathEvent {
                            path: relative.clone(),
                        });
                        recip.do_send(FsEvent {
                            kind: evt,
                            ctx: self.ctx.clone(),
                            span: None,
                        })
                    }
                }
                Err(e) => {
                    tracing::debug!(?e, "could not remove");
                }
            }
        }
        Ok(())
    }
}
