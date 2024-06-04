use crate::actor::FsWatcher;
use actix::{ActorContext, Handler};
use std::sync::Arc;
use tracing::{instrument, Span};

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct StopWatcher(pub Arc<Span>);

impl Handler<StopWatcher> for FsWatcher {
    type Result = ();

    #[instrument(skip_all, name = "StopWatcher for FsWatcher", parent=msg.0.id())]
    fn handle(&mut self, msg: StopWatcher, ctx: &mut Self::Context) -> Self::Result {
        self.watcher = None;
        ctx.stop();
    }
}
