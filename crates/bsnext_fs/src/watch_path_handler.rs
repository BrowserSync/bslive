use crate::actor::FsWatcher;
use crate::{FsEvent, FsEventKind, PathAddedEvent, PathEvent};
use actix::{ActorContext, Handler, Recipient};
use notify::{RecursiveMode, Watcher};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::{trace_span, Span};

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct RequestWatchPath {
    pub recipients: Vec<Recipient<FsEvent>>,
    pub path: PathBuf,
    pub span: Arc<Span>,
}

impl Handler<RequestWatchPath> for FsWatcher {
    type Result = ();

    // todo: ensure this isn't sent for every input change
    fn handle(&mut self, msg: RequestWatchPath, _ctx: &mut Self::Context) -> Self::Result {
        let span = trace_span!(parent: msg.span.id(), "RequestWatchPath for FsWatcher", ?msg.path);
        let s = Arc::new(span);
        let _guard = s.enter();
        // tracing::trace!(path = ?msg.path, "-> WatchPath");
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
                    for recip in &msg.recipients {
                        let evt = FsEventKind::PathNotFoundError(PathEvent {
                            path: msg.path.clone(),
                        });
                        recip.do_send(FsEvent {
                            kind: evt,
                            ctx: self.ctx.clone(),
                        })
                    }
                    _ctx.stop();
                }
            }
        }
    }
}
