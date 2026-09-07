#![allow(clippy::result_large_err)]
use crate::api::BsSystemApi;
use crate::start::SystemStart;
use crate::system::BsSystem;
use actix::{ActorContext, Handler};
use bsnext_core::shared_args::InputOpts;
use bsnext_dto::any_event::AnyEvent;
use bsnext_dto::StartupError;
use std::path::PathBuf;

pub async fn start_system(
    cwd: PathBuf,
    system_start: impl SystemStart,
    input_opts: InputOpts,
    events_sender: tokio::sync::mpsc::Sender<AnyEvent>,
) -> Result<Option<BsSystemApi>, StartupError> {
    match system_start
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
