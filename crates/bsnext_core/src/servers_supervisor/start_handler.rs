use crate::server::error::{PatchError, ServerError};
use crate::servers_supervisor::actor::{ChildHandler, ServersSupervisor};
use bsnext_input::server_config::Identity;

#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct ChildCreated {
    pub server_handler: ChildHandler,
}
#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct ChildPatched {
    pub server_handler: ChildHandler,
}
#[derive(Debug)]
pub enum ChildResult {
    Created(ChildCreated),
    Patched(ChildPatched),
    PatchErr(ChildNotPatched),
    Stopped(Identity),
    Err(ChildNotCreated),
}

impl actix::Handler<ChildCreated> for ServersSupervisor {
    type Result = ();

    fn handle(&mut self, msg: ChildCreated, _ctx: &mut Self::Context) -> Self::Result {
        self.handlers.insert(
            msg.server_handler.identity.clone(),
            msg.server_handler.clone(),
        );
        tracing::trace!("ChildCreated child count: {}", self.handlers.len());
    }
}

#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct ChildNotCreated {
    pub server_error: ServerError,
    pub identity: Identity,
}

#[derive(Debug, actix::Message)]
#[rtype(result = "()")]
pub struct ChildNotPatched {
    pub patch_error: PatchError,
    pub identity: Identity,
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
