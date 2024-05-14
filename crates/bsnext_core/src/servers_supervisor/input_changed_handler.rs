use crate::dto::ServerChangeSet;
use crate::servers_supervisor::actor::ServersSupervisor;
use actix::AsyncContext;
use bsnext_input::Input;
use std::future::Future;

use std::pin::Pin;

#[derive(actix::Message)]
#[rtype(result = "ServerChangeSet")]
pub struct InputChanged {
    pub input: Input,
}

impl actix::Handler<InputChanged> for ServersSupervisor {
    type Result = Pin<Box<dyn Future<Output = ServerChangeSet>>>;

    fn handle(&mut self, msg: InputChanged, ctx: &mut Self::Context) -> Self::Result {
        self.input_changed(ctx.address(), msg.input)
    }
}
