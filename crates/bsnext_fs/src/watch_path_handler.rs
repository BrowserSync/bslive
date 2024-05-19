use crate::actor::FsWatcher;
use crate::{FsEvent, FsEventKind, FsWatchError, PathAddedEvent};
use actix::{ActorContext, Handler, Recipient};
use notify::{RecursiveMode, Watcher};
use std::path::PathBuf;

#[derive(actix::Message)]
#[rtype(result = "Result<(), FsWatchError>")]
pub struct RequestWatchPath {
    pub recipients: Vec<Recipient<FsEvent>>,
    pub path: PathBuf,
}

impl Handler<RequestWatchPath> for FsWatcher {
    type Result = Result<(), FsWatchError>;

    // todo: ensure this isn't sent for every input change
    fn handle(&mut self, msg: RequestWatchPath, _ctx: &mut Self::Context) -> Self::Result {
        tracing::trace!(path = ?msg.path, "-> WatchPath");
        if let Some(watcher) = self.watcher.as_mut() {
            match watcher.watch(&msg.path, RecursiveMode::Recursive) {
                Ok(_) => {
                    let new_recipients = msg
                        .recipients
                        .into_iter()
                        .filter(|r| !self.receivers.contains(r))
                        .collect::<Vec<_>>();
                    self.receivers.extend(new_recipients);

                    tracing::debug!(path = ?msg.path, "👀 watching! {} receivers", self.receivers.len());
                    tracing::debug!(?self.cwd);
                    tracing::debug!("{} receivers", self.receivers.len());

                    let matched = msg.path == self.cwd;

                    let relative = if matched {
                        msg.path.clone()
                    } else {
                        match msg.path.strip_prefix(&self.cwd) {
                            Ok(stripped) => stripped.to_path_buf(),
                            Err(e) => {
                                tracing::debug!(?e, "could not extract the CWD from a path");
                                msg.path.clone()
                            }
                        }
                    };
                    for recip in &self.receivers {
                        let evt = FsEventKind::PathAdded(PathAddedEvent {
                            path: relative.clone(),
                            debounce: self.debounce,
                        });
                        recip.do_send(FsEvent {
                            kind: evt,
                            ctx: self.ctx.clone(),
                        })
                    }
                }
                Err(err) => {
                    tracing::error!("cannot watch: {}", err);
                    _ctx.stop();
                    return Err(FsWatchError::Watcher(err));
                }
            }
        }
        Ok(())
    }
}
