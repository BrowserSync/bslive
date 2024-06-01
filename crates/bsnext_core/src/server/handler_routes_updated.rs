use crate::server::actor::ServerActor;
use crate::server::handler_change::Change;
use std::sync::Arc;

use bsnext_dto::ClientEvent;
use bsnext_input::route_manifest::RouteChangeSet;
use tracing::Span;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct RoutesUpdated {
    pub change_set: RouteChangeSet,
    pub span: Arc<Span>,
}

impl actix::Handler<RoutesUpdated> for ServerActor {
    type Result = ();

    #[tracing::instrument(skip_all, name="RoutesUpdated for ServerActor", parent=msg.span.id())]
    fn handle(&mut self, msg: RoutesUpdated, _ctx: &mut Self::Context) -> Self::Result {
        let Some(client_sender) = self.signals.as_ref().and_then(|s| s.client_sender.as_ref())
        else {
            return tracing::error!("signals not ready, should they be?");
        };

        // todo(alpha): check if an event can be removed now
        let mut outgoing = vec![];
        for route_id in &msg.change_set.changed {
            outgoing.push(Change::fs(&route_id.path))
        }
        for route_id in &msg.change_set.removed {
            outgoing.push(Change::fs_removed(&route_id.path))
        }
        for route_id in &msg.change_set.added {
            outgoing.push(Change::fs_added(&route_id.path))
        }
        tracing::debug!(?outgoing, "outgoing messages");
        for change in outgoing {
            match client_sender.send(ClientEvent::Change(change.into())) {
                Ok(r) => {
                    tracing::trace!(?r, "change event sent to clients");
                }
                Err(_) => tracing::error!("not sent to client_sender"),
            };
        }
    }
}
