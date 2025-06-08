use crate::start::start_kind::StartKind;
use crate::start::start_system::start_system;
use bsnext_dto::internal::{AnyEvent, InternalEvents};
use bsnext_output::stdout::StdoutTarget;
use bsnext_output::OutputWriters;
use std::future::Future;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

pub mod start_command;
pub mod start_kind;
pub mod start_system;

pub fn stdout_channel(writer: OutputWriters) -> (Sender<AnyEvent>, impl Future<Output = ()>) {
    let (events_sender, mut events_receiver) = mpsc::channel::<AnyEvent>(1);
    let channel_future = async move {
        let stdout = &mut std::io::stdout();
        let stderr = &mut std::io::stderr();
        let mut sink = StdoutTarget::new(stdout, stderr);
        while let Some(evt) = events_receiver.recv().await {
            tracing::trace!(?evt);
            let result = match evt {
                AnyEvent::Internal(int) => writer.write_evt(&int, &mut sink.output()),
                AnyEvent::External(ext) => writer.write_evt(&ext, &mut sink.output()),
            };
            match result {
                Ok(_) => {}
                Err(_) => tracing::error!("could not handle event"),
            }
            sink.flush();
        }
    };
    (events_sender, channel_future)
}

pub async fn with_sender(
    cwd: PathBuf,
    start_kind: StartKind,
    events_sender: Sender<AnyEvent>,
) -> Result<(), anyhow::Error> {
    let ecc = events_sender.clone();

    let startup = start_system(cwd, start_kind, events_sender).await;
    match startup {
        // If the startup was successful, keep hold of the handle to keep the system running
        Ok(api) => match api.handle.await {
            Ok(..) => Ok(()),
            Err(er) => Err(anyhow::anyhow!("{}", er)),
        },
        Err(err) => {
            let as_str = err.to_string();
            let _ = ecc
                .send(AnyEvent::Internal(InternalEvents::StartupError(err)))
                .await;
            Err(anyhow::anyhow!("{}", as_str))
        }
    }
}
