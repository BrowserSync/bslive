use crate::actor::FsWatcher;
use crate::filter::{Filter, PathFilter};
use crate::{
    BufferedChangeEvent, ChangeEvent, FsEvent, FsEventKind, PathDescription, PathDescriptionOwned,
};
use actix::Handler;
use std::collections::BTreeSet;
use std::path::PathBuf;
use std::time::Instant;

#[derive(actix::Message, Hash, PartialEq, Eq, Ord, PartialOrd, Debug, Clone)]
#[rtype(result = "()")]
pub struct InnerChangeEvent {
    pub absolute_path: PathBuf,
}

impl Handler<InnerChangeEvent> for FsWatcher {
    type Result = ();
    fn handle(&mut self, msg: InnerChangeEvent, _ctx: &mut Self::Context) -> Self::Result {
        tracing::debug!(?self.ctx, "InnerChangeEvent for FsWatcher");
        tracing::debug!("  └ sending to {} receivers", self.receivers.len());
        let relative = match msg.absolute_path.strip_prefix(&self.cwd) {
            Ok(stripped) => stripped.to_path_buf(),
            Err(e) => {
                tracing::trace!(?e, "could not extract the CWD from a path");
                msg.absolute_path.clone()
            }
        };
        for x in &self.receivers {
            x.do_send(FsEvent {
                kind: FsEventKind::Change(ChangeEvent {
                    absolute_path: msg.absolute_path.clone(),
                    path: relative.clone(),
                }),
                fs_event_ctx: self.ctx.clone(),
            })
        }
    }
}

#[derive(actix::Message, Hash, PartialEq, Eq, Ord, PartialOrd, Debug, Clone)]
#[rtype(result = "()")]
pub struct MultipleInnerChangeEvent {
    pub events: Vec<InnerChangeEvent>,
}

impl Handler<MultipleInnerChangeEvent> for FsWatcher {
    type Result = ();
    fn handle(&mut self, msg: MultipleInnerChangeEvent, _ctx: &mut Self::Context) -> Self::Result {
        tracing::debug!(?self.ctx, "MultipleInnerChangeEvent for FsWatcher");
        tracing::debug!("  └ got {} events to process", msg.events.len());
        tracing::debug!(
            "  └ will apply {} filter & {} ignore",
            self.filters.len(),
            self.ignore.len()
        );
        let now = Instant::now();
        let unique = msg.events.iter().collect::<BTreeSet<_>>();
        let original_len = msg.events.len();
        let unique_len = unique.len();
        tracing::debug!("  └ {} unique event after converting to set", unique.len());
        tracing::debug!("  └ {:?}", unique);

        let filtered = unique
            .iter()
            .map(|inner| PathDescription {
                absolute: &inner.absolute_path,
                relative: inner.absolute_path.strip_prefix(&self.cwd).ok(),
            })
            .filter(|pd| {
                if self.filters.is_empty() {
                    true
                } else {
                    self.filters.iter().any(|filter| filter.filter(pd))
                }
            })
            .filter(|pd| {
                if self.ignore.is_empty() {
                    true
                } else {
                    let path_matched_ignore_filter =
                        self.ignore.iter().any(|filter| filter.filter(pd));
                    if path_matched_ignore_filter {
                        false
                    } else {
                        true
                    }
                }
            })
            .collect::<Vec<PathDescription>>();

        tracing::debug!("  └ accepted {} after filtering", filtered.len());

        let time = Instant::now() - now;
        tracing::debug!("took {}ms to process event", time.as_millis());

        if filtered.is_empty() {
            tracing::debug!("no changes to send. original: {original_len}, filtered: {unique_len}");
            return;
        }

        for recipient in &self.receivers {
            let evt = FsEventKind::ChangeBuffered(BufferedChangeEvent {
                // this might look expensive, but in reality not expecting more than 1 receiver
                events: filtered.iter().map(PathDescriptionOwned::from).collect(),
            });
            recipient.do_send(FsEvent {
                kind: evt,
                fs_event_ctx: self.ctx.clone(),
            })
        }
    }
}
