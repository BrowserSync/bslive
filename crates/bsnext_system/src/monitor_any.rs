use crate::system::BsSystem;
use actix::{ActorFutureExt, AsyncContext};
use actix::{ResponseActFuture, WrapFuture};
use bsnext_input::Input;
use bsnext_monitor::watchables::accept_watchables;
use tracing::debug_span;

/// Message to monitor file system paths based on the current input configuration.
///
/// This handler processes an `Input` configuration to determine which paths should be
/// monitored for file system changes. It:
/// - Converts the input into a collection of watchables based on the configured watch strategy
///   (routes, servers, or any custom watchers)
/// - Sends these watchables to the `Monitor` actor to create individual `PathMonitor` instances
/// - Registers the resulting task specifications with the system for later use in handling
///   file system events
///
/// The monitoring configuration respects the `watchers` and `infer` settings in the input,
/// allowing for flexible control over which files and directories are watched.
#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct MonitorAny {
    input: Input,
}

impl MonitorAny {
    pub fn new(input: Input) -> Self {
        Self { input }
    }
}

impl actix::Handler<MonitorAny> for BsSystem {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: MonitorAny, ctx: &mut Self::Context) -> Self::Result {
        let cwd = self.cwd.clone();
        let recip = ctx.address().recipient();
        let monitor = self.monitor.clone();

        let future = async move {
            let input = msg.input;
            let msg = accept_watchables(cwd, &input, recip);
            monitor.send(msg).await?;
            Ok::<_, anyhow::Error>(())
        };

        Box::pin(future.into_actor(self).map(|res, _actor, _ctx| match res {
            Ok(..) => {
                let span = debug_span!("MonitorAny -> InsertResults");
                let _g = span.entered();
                tracing::debug!("a monitor was added");
                // for (index, insert_result) in inner.into_iter().enumerate() {
                // let task_spec = TaskSpec::opt_from(&insert_result.meta.watch_spec);
                // let fs_ctx = insert_result.meta.fs_ctx;
                // actor.insert_meta(insert_result.meta.fs_ctx, insert_result.meta);
                // if let Some(spec) = task_spec {
                //     actor.insert_task_spec(fs_ctx, spec);
                // }
                // }
            }
            Err(err) => {
                tracing::error!(?err);
            }
        }))
    }
}
