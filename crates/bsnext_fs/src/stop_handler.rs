use crate::actor::FsWatcher;
use actix::{ActorContext, Handler};

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
pub struct StopWatcher;

impl Handler<StopWatcher> for FsWatcher {
    type Result = ();

    fn handle(&mut self, _msg: StopWatcher, ctx: &mut Self::Context) -> Self::Result {
        self.watcher = None;
        ctx.stop();
    }
}
