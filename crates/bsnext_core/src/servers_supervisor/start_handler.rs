use crate::servers_supervisor::actor::{ChildHandler, ServersSupervisor};
use bsnext_dto::internal::ChildNotCreated;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct ChildCreatedInsert {
    pub child_handler: ChildHandler,
}

impl actix::Handler<ChildCreatedInsert> for ServersSupervisor {
    type Result = ();

    fn handle(&mut self, msg: ChildCreatedInsert, _ctx: &mut Self::Context) -> Self::Result {
        self.handlers.insert(
            msg.child_handler.identity.clone(),
            msg.child_handler.clone(),
        );
        tracing::trace!("ChildCreated child count: {}", self.handlers.len());
    }
}

impl actix::Handler<ChildNotCreated> for ServersSupervisor {
    type Result = ();

    fn handle(&mut self, _msg: ChildNotCreated, _ctx: &mut Self::Context) -> Self::Result {
        // self.handlers.insert(
        //     msg.server_handler.identity.clone(),
        //     msg.server_handler.clone(),
        // );
        // tracing::trace!("ChildCreated child count: {}", self.handlers.len());
    }
}
