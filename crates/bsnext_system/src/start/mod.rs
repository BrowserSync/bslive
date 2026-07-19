use crate::start::start_kind::StartKind;
use crate::start::start_system::start_system;
use bsnext_dto::any_event::AnyEvent;
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::{DidStart, StartupError, StartupErrorDTO};
use bsnext_input::startup::{StartupContext, SystemStartArgs};
use bsnext_input::InputError;
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
            tracing::trace!(parent: None, ?evt, "stdout_channel recv()");
            let result = match evt {
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

#[tracing::instrument]
pub async fn with_sender(
    cwd: PathBuf,
    start_kind: StartKind,
    events_sender: Sender<AnyEvent>,
) -> Result<(), anyhow::Error> {
    let ecc = events_sender.clone();

    let startup = start_system(cwd, start_kind, events_sender).await;
    match startup {
        // If the startup was successful, keep hold of the handle to keep the system running
        Ok(Some(api)) => match api.handle().await {
            Ok(..) => Ok(()),
            Err(er) => Err(anyhow::anyhow!("{}", er)),
        },
        Ok(None) => Ok(()),
        Err(err) => {
            let as_str = err.to_string();
            let _ = ecc
                .send(AnyEvent::External(ExternalEventsDTO::StartupError(
                    StartupErrorDTO {
                        error: as_str.clone(),
                    },
                )))
                .await;
            Err(anyhow::anyhow!("{}", as_str))
        }
    }
}

pub trait SystemStart {
    fn resolve_input(&self, ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>>;
    fn start(&self, _ctx: &StartupContext) -> impl Future<Output = Result<DidStart, StartupError>> {
        futures::future::ready(Ok(DidStart::WillExit))
    }
}
