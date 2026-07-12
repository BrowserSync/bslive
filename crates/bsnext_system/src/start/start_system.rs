#![allow(clippy::result_large_err)]
use crate::api::BsSystemApi;
use crate::start::SystemStart;
use crate::system::BsSystem;
use actix::{Actor, ActorContext, Handler};
use bsnext_core::shared_args::InputOpts;
use bsnext_dto::any_event::AnyEvent;
use bsnext_dto::StartupError;
use std::path::PathBuf;

pub async fn start_system(
    cwd: PathBuf,
    start_kind: impl SystemStart,
    input_opts: InputOpts,
    events_sender: tokio::sync::mpsc::Sender<AnyEvent>,
) -> Result<Option<BsSystemApi>, StartupError> {
    match start_kind
        .start(cwd, input_opts, events_sender.clone())
        .await
    {
        Ok(DidStart::Started { api }) => {
            tracing::debug!("DidStart::Started");
            Ok(Some(api))
        }
        Ok(DidStart::WillExit) => {
            tracing::debug!("DidStart::WillExit");
            Ok(None)
        }
        Err(e) => {
            let message = e.to_string();
            Err(StartupError::Other(message))
        }
    }
}

// #[derive(Debug, actix::Message)]
// #[rtype(result = "Result<DidStart, StartupError>")]
// pub struct Start {
//     pub kind: StartKind,
// }

// impl Handler<Start> for BsSystem {
//     type Result = ResponseActFuture<Self, Result<DidStart, StartupError>>;
//
//     #[tracing::instrument(name = "BsSystem->Start", skip(self, msg, ctx))]
//     fn handle(&mut self, msg: Start, ctx: &mut Self::Context) -> Self::Result {
//         let addr = ctx.address();
//         let servers_addr = self.servers().clone();
//         let StartKind::FromPaths(paths) = msg.kind else {
//             todo!("not ready!");
//         };
//         let ctx = self.start_context.clone();
//         let f = async move {
//             let output = paths.start().await;
//             dbg!(&"did get");
//             output
//         };
//         Box::pin(f.into_actor(self))
//     }
// }

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct StopSystem;

impl Handler<StopSystem> for BsSystem {
    type Result = ();

    fn handle(&mut self, _msg: StopSystem, ctx: &mut Self::Context) -> Self::Result {
        tracing::trace!("handling StopSystem. Note: not graceful.");
        ctx.stop();
    }
}

#[derive(Debug)]
pub enum DidStart {
    Started { api: BsSystemApi },
    WillExit,
}
