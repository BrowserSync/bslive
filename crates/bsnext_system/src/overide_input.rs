use crate::servers::ResolveServers;
use crate::BsSystem;
use actix::{ActorFutureExt, AsyncContext, ResponseActFuture, WrapFuture};
use bsnext_dto::internal::{AnyEvent, ChildResult, ServerError};
use bsnext_dto::GetActiveServersResponse;
use bsnext_input::startup::StartupContext;
use bsnext_input::{Input, InputCtx};
use tracing::debug;

#[derive(Debug, actix::Message)]
#[rtype(result = "Result<(GetActiveServersResponse, Vec<ChildResult>), ServerError>")]
pub struct OverrideInput {
    pub input: Input,
    pub original_event: AnyEvent,
}

impl actix::Handler<OverrideInput> for BsSystem {
    type Result =
        ResponseActFuture<Self, Result<(GetActiveServersResponse, Vec<ChildResult>), ServerError>>;

    fn handle(&mut self, msg: OverrideInput, ctx: &mut Self::Context) -> Self::Result {
        let input_clone = msg.input.clone();
        let start_ctx_clone = self
            .start_context
            .clone()
            .expect("If we get here, it's a big problem");
        // let ctx_clone = self.st
        let f = ctx
            .address()
            .send(ResolveServers { input: msg.input })
            .into_actor(self)
            .map(move |res, actor, _ctx| {
                debug!(" + did override input");
                let output = match res {
                    Ok(Ok(res)) => Ok(res),
                    Ok(Err(s_e)) => Err(s_e),
                    Err(err) => Err(ServerError::Unknown(err.to_string())),
                };
                actor.accept_watchables(&input_clone);
                actor.update_ctx(&input_clone, &start_ctx_clone);
                output
            });
        Box::pin(f)
    }
}

impl BsSystem {
    fn update_ctx(&mut self, input: &Input, ctx: &StartupContext) {
        let next = input
            .servers
            .iter()
            .map(|s| s.identity.clone())
            .collect::<Vec<_>>();

        if let Some(mon) = self.input_monitors.as_mut() {
            let next_input_ctx = InputCtx::new(&next, None, ctx, mon.input_ctx.file_path());
            if !next.is_empty() {
                if next_input_ctx == mon.input_ctx {
                    tracing::info!(
                        " - server identities were equal, not updating ctx {:?}",
                        next_input_ctx
                    );
                } else {
                    tracing::info!(
                        " + updating stored server identities following a file change {:?}",
                        next
                    );
                    mon.input_ctx = next_input_ctx
                }
            }
        }
    }
}
