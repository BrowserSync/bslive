use crate::args::Args;
use crate::start_kind::StartKind;
use crate::{BsSystem, Start};
use actix::Actor;
use bsnext_dto::internal::{AnyEvent, StartupEvent};
use bsnext_input::startup::{DidStart, StartupError};
use bsnext_output::OutputWriterTrait;
use std::path::PathBuf;
use tokio::sync::oneshot;

pub async fn start_cmd(
    cwd: PathBuf,
    args: Args,
    events_sender: tokio::sync::mpsc::Sender<AnyEvent>,
) -> Result<(), impl OutputWriterTrait> {
    let (tx, rx) = oneshot::channel();
    let system = BsSystem::new();
    let sys_addr = system.start();
    let start_kind = StartKind::from_args(&args);

    tracing::debug!(?start_kind);

    let start = Start {
        kind: start_kind,
        cwd: Some(cwd),
        ack: tx,
        events_sender,
    };

    match sys_addr.send(start).await {
        Ok(Ok(DidStart::Started)) => {
            // everything good here, continue...
        }
        Ok(Err(e)) => {
            return Err(StartupEvent::FailedStartup(e));
        }
        Err(e) => {
            let message = e.to_string();
            return Err(StartupEvent::FailedStartup(StartupError::Other(message)));
        }
    };

    match rx.await {
        Ok(_) => {
            tracing::info!("servers ended");
        }
        Err(e) => {
            // dropped? this is ok
            tracing::trace!(?e, "");
        }
    };

    Ok(())
}
