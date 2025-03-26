use crate::servers_supervisor::actor::ServersSupervisor;
use actix::{Addr, AsyncContext};
use bsnext_input::Input;
use std::future::Future;

use crate::server::actor::ServerActor;
use bsnext_dto::internal::ChildResult;
use std::pin::Pin;

#[derive(actix::Message)]
#[rtype(result = "InputChangedResponse")]
pub struct InputChanged {
    pub input: Input,
}

#[derive(Debug)]
pub struct InputChangedResponse {
    pub changes: Vec<(Option<Addr<ServerActor>>, ChildResult)>,
}

impl InputChangedResponse {
    pub fn from_changes(changes: Vec<(Option<Addr<ServerActor>>, ChildResult)>) -> Self {
        Self { changes }
    }
}

impl actix::Handler<InputChanged> for ServersSupervisor {
    type Result = Pin<Box<dyn Future<Output = InputChangedResponse>>>;

    fn handle(&mut self, msg: InputChanged, ctx: &mut Self::Context) -> Self::Result {
        self.input_changed(ctx.address(), msg.input)
    }
}
