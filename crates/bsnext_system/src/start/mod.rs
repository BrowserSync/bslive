use crate::start::start_system::start_system;
use crate::system::BsSystem;
use actix::Addr;
use bsnext_core::shared_args::InputOpts;
use bsnext_dto::any_event::AnyEvent;
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::{StartupError, StartupErrorDTO};
use bsnext_output::stdout::StdoutTarget;
use bsnext_output::OutputWriters;
use start_system::DidStart;
use std::future::Future;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;

pub mod start_command;
pub mod start_from_paths;
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

#[tracing::instrument(skip(system_start))]
pub async fn with_sender(
    cwd: PathBuf,
    system_start: impl SystemStart,
    input_opts: InputOpts,
    events_sender: Sender<AnyEvent>,
) -> Result<(), anyhow::Error> {
    let ecc = events_sender.clone();

    let startup = start_system(cwd, system_start, input_opts, events_sender).await;
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
    #[allow(async_fn_in_trait)]
    async fn start(
        &self,
        _cwd: PathBuf,
        _input_opts: InputOpts,
        _sink: tokio::sync::mpsc::Sender<AnyEvent>,
    ) -> Result<DidStart, StartupError> {
        Ok(DidStart::WillExit)
    }
    #[allow(async_fn_in_trait)]
    async fn addr(&self) -> Addr<BsSystem> {
        // self.sys.addr()
        todo!("impl!")
    }
}
