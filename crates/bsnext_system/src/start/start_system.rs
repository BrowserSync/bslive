use crate::start::start_kind::StartKind;
use crate::{BsSystem, BsSystemApi, Start};
use actix::Actor;
use bsnext_dto::internal::AnyEvent;
use bsnext_dto::{DidStart, StartupError};
use std::path::PathBuf;
use tokio::sync::oneshot;

pub async fn start_system(
    cwd: PathBuf,
    start_kind: StartKind,
    events_sender: tokio::sync::mpsc::Sender<AnyEvent>,
) -> Result<BsSystemApi, StartupError> {
    let (tx, rx) = oneshot::channel();
    let system = BsSystem::new();
    let sys_addr = system.start();

    tracing::debug!("{:#?}", start_kind);

    let start = Start {
        kind: start_kind,
        cwd,
        ack: tx,
        events_sender,
    };

    match sys_addr.send(start).await {
        Ok(Ok(DidStart::Started(..))) => {
            let api = BsSystemApi {
                sys_address: sys_addr,
                handle: rx,
            };
            Ok(api)
        }
        Ok(Err(e)) => Err(e),
        Err(e) => {
            let message = e.to_string();
            Err(StartupError::Other(message))
        }
    }
}
