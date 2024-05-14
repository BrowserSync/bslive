use crate::server::actor::ServerActor;
use crate::server::handler_change::Change;
use actix::AsyncContext;
use bsnext_input::route_manifest::RouteChangeSet;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct RoutesUpdated {
    pub change_set: RouteChangeSet,
}

impl actix::Handler<RoutesUpdated> for ServerActor {
    type Result = ();

    fn handle(&mut self, msg: RoutesUpdated, ctx: &mut Self::Context) -> Self::Result {
        tracing::trace!(changeset=?msg.change_set, id = ?self.config.identity);
        for id in &msg.change_set.changed {
            ctx.address().do_send(Change::fs(&id.path))
        }
        for id in &msg.change_set.removed {
            ctx.address().do_send(Change::fs_removed(&id.path))
        }
        for id in &msg.change_set.added {
            ctx.address().do_send(Change::fs_added(&id.path))
        }
    }
}
