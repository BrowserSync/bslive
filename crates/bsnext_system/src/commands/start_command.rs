use crate::args::Args;
use crate::start_kind::StartKind;
use crate::{BsSystem, Start};
use actix::Actor;
use bsnext_dto::internal::{AnyEvent, StartupEvent};
use bsnext_input::startup::{DidStart, StartupError};
use bsnext_output::stdout::StdoutTarget;
use bsnext_output::{OutputWriterTrait, OutputWriters};
use bsnext_tracing::OutputFormat;
use std::path::PathBuf;
use tokio::sync::{mpsc, oneshot};
use tracing::debug_span;

pub async fn start_cmd(cwd: PathBuf, args: Args) -> Result<(), impl OutputWriterTrait> {
    let (tx, rx) = oneshot::channel();
    let (events_sender, mut events_receiver) = mpsc::channel::<AnyEvent>(1);

    let system = BsSystem::new();
    let sys_addr = system.start();
    let format_clone = args.format;
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

    let events_handler = tokio::spawn(async move {
        // at this point, we started, so we can choose a TUI
        let printer = match format_clone {
            OutputFormat::Tui => {
                // let rr = Ratatui::try_new().expect("test");
                // let (sender, _ui_handle, _other) = rr.install().expect("thread install");
                // Writers::Ratatui(sender)
                todo!("re-implement ratatui")
            }
            OutputFormat::Json => OutputWriters::Json,
            OutputFormat::Normal => OutputWriters::Pretty,
        };
        // let events = vec![];
        let stdout = &mut std::io::stdout();
        let stderr = &mut std::io::stderr();
        let mut sink = StdoutTarget::new(stdout, stderr);

        while let Some(evt) = events_receiver.recv().await {
            let span = debug_span!("External Event processor");
            let _g2 = span.enter();
            tracing::debug!(external_event = ?evt);
            let result = match evt {
                AnyEvent::Internal(int) => printer.write_evt(&int, &mut sink.output()),
                AnyEvent::External(ext) => printer.write_evt(&ext, &mut sink.output()),
            };
            match result {
                Ok(_) => {}
                Err(_) => tracing::error!("could not handle event"),
            }
            sink.flush();
        }
    });

    match rx.await {
        Ok(_) => {
            tracing::info!("servers ended");
        }
        Err(e) => {
            // dropped? this is ok
            tracing::trace!(?e, "");
        }
    };

    match events_handler.await {
        Ok(_) => {
            tracing::info!("events ended");
        }
        Err(e) => {
            // dropped? this is ok
            tracing::trace!(?e, "");
        }
    }

    Ok(())
}
