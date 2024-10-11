use crate::args::Args;
use crate::start_kind::StartKind;
use crate::{AnyEvent, BsSystem, Start};
use actix::Actor;

use bsnext_dto::internal::StartupEvent;
use bsnext_input::startup::DidStart;
use bsnext_output::ratatui::Ratatui;
use bsnext_output::{OutputWriter, Writers};
use bsnext_tracing::{init_tracing, OtelOption, OutputFormat, WriteOption};
use clap::Parser;
use std::env::current_dir;
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use tokio::sync::{mpsc, oneshot};
use tracing::debug_span;

/// The typical lifecycle when ran from a CLI environment
pub async fn from_args<I, T>(itr: I) -> Result<(), anyhow::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    std::env::set_var("RUST_LIB_BACKTRACE", "0");
    let cwd = PathBuf::from(current_dir().unwrap().to_string_lossy().to_string());
    let args = Args::parse_from(itr);
    let format_clone = args.format;
    let write_opt = if args.write_log {
        WriteOption::File
    } else {
        WriteOption::None
    };

    let otel = if args.otel {
        OtelOption::On
    } else {
        OtelOption::Off
    };
    let _g = init_tracing(args.log_level, args.format, write_opt, otel);
    tracing::debug!("{:#?}", args);

    let (tx, rx) = oneshot::channel();
    let (events_sender, mut events_receiver) = mpsc::channel::<AnyEvent>(1);

    let system = BsSystem::new();
    let sys_addr = system.start();

    let start_kind = StartKind::from_args(args);

    let start = Start {
        kind: start_kind,
        cwd: Some(cwd),
        ack: tx,
        events_sender,
    };

    // for the startup message, don't allow a TUI yet
    let start_printer = match format_clone {
        OutputFormat::Tui => Writers::Pretty,
        OutputFormat::Json => Writers::Json,
        OutputFormat::Normal => Writers::Pretty,
    };

    let stdout = &mut std::io::stdout();

    match sys_addr.send(start).await? {
        Ok(DidStart::Started) => {
            let evt = StartupEvent::Started;
            match start_printer.handle_startup_event(stdout, &evt) {
                Ok(_) => {}
                Err(e) => tracing::error!(?e),
            };
            match stdout.flush() {
                Ok(_) => {}
                Err(e) => tracing::error!("could not flush {e}"),
            };
        }
        Err(e) => {
            let evt = StartupEvent::FailedStartup(e);
            match start_printer.handle_startup_event(stdout, &evt) {
                Ok(_) => {}
                Err(e) => tracing::error!(?e),
            };
            match stdout.flush() {
                Ok(_) => {}
                Err(e) => tracing::error!("could not flush {e}"),
            };
            return Err(anyhow::anyhow!("could not flush"));
        }
    };

    // at this point, we started, so we can choose a TUI
    let printer = match format_clone {
        OutputFormat::Tui => {
            let rr = Ratatui::try_new().expect("test");
            let (sender, _ui_handle, _other) = rr.install().expect("thread install");
            Writers::Ratatui(sender)
        }
        OutputFormat::Json => Writers::Json,
        OutputFormat::Normal => Writers::Pretty,
    };

    let events_handler = tokio::spawn(async move {
        // let events = vec![];
        let stdout = &mut std::io::stdout();

        while let Some(evt) = events_receiver.recv().await {
            let span = debug_span!("External Event processor");
            let _g2 = span.enter();
            tracing::debug!(external_event=?evt);
            let r = match evt {
                AnyEvent::Internal(int) => printer.handle_internal_event(stdout, int),
                AnyEvent::External(ext) => printer.handle_external_event(stdout, &ext),
            };
            match stdout.flush() {
                Ok(_) => {}
                Err(e) => tracing::error!("could not flush {e}"),
            };
            match r {
                Ok(_) => {}
                Err(_) => tracing::error!("could not handle event"),
            }
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

    // match ui_handle.join() {
    //     Ok(_) => {}
    //     Err(e) => tracing::error!(?e),
    // };

    // match events_handler.await {
    //     Ok(v) => {
    //         tracing::info!(?v, "events seen");
    //         let errors = v
    //             .iter()
    //             .filter(|e| matches!(e, ExternalEvents::InputError(..)))
    //             .collect::<Vec<_>>();
    //         if !errors.is_empty() {
    //             tracing::info!("stopped for the following reasons");
    //             for msg in errors {
    //                 tracing::error!(?msg);
    //             }
    //             return Err(anyhow::anyhow!("exited..."));
    //         }
    //     }
    //     Err(e) => tracing::error!(?e),
    // }
    Ok(())
}
