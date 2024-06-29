use crate::servers_supervisor::actor::ServersSupervisor;
use actix::{Addr, AsyncContext};
use bsnext_input::Input;
use std::future::Future;

use crate::server::actor::ServerActor;
use bsnext_dto::internal::ChildResult;
use std::pin::Pin;

#[derive(actix::Message)]
#[rtype(result = "Vec<(Option<Addr<ServerActor>>, ChildResult)>")]
pub struct InputChanged {
    pub input: Input,
}

impl actix::Handler<InputChanged> for ServersSupervisor {
    type Result = Pin<Box<dyn Future<Output = Vec<(Option<Addr<ServerActor>>, ChildResult)>>>>;

    #[tracing::instrument(skip_all, name = "InputChanged for ServersSupervisor")]
    fn handle(&mut self, msg: InputChanged, ctx: &mut Self::Context) -> Self::Result {
        self.input_changed(ctx.address(), msg.input)
    }
}
