use crate::args::Args;
use crate::start_kind::StartKind;
use crate::{BsSystem, EventWithSpan, Start};
use actix::Actor;

use bsnext_dto::{ExternalEvents, StartupEvent};
use bsnext_input::startup::{DidStart, StartupResult};
use bsnext_output::ratatui::Ratatui;
use bsnext_output::{OutputWriter, Writers};
use bsnext_tracing::{init_tracing, OtelOption, OutputFormat, WriteOption};
use clap::Parser;
use std::env::current_dir;
use std::ffi::OsString;
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::thread;
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
    let (startup_oneshot_sender, startup_oneshot_receiver) = oneshot::channel::<StartupResult>();
    let (events_sender, mut events_receiver) = mpsc::channel::<ExternalEvents>(1);

    let system = BsSystem::new();
    let sys_addr = system.start();

    let start_kind = StartKind::from_args(args);

    let start = Start {
        kind: start_kind,
        cwd: Some(cwd),
        ack: tx,
        events_sender,
        startup_oneshot_sender,
    };

    // match startup_oneshot_receiver.await? {
    //     Ok(DidStart::Started) => {
    //         let evt = StartupEvent::Started;
    //         // match printer.handle_startup_event(stdout, &evt) {
    //         //     Ok(_) => {}
    //         //     Err(e) => tracing::error!(?e),
    //         // };
    //         // match stdout.flush() {
    //         //     Ok(_) => {}
    //         //     Err(e) => tracing::error!("could not flush {e}"),
    //         // };
    //     }
    //     Err(e) => {
    //         let evt = StartupEvent::FailedStartup((&e).into());
    //         match printer.handle_startup_event(stdout, &evt) {
    //             Ok(_) => {}
    //             Err(e) => tracing::error!(?e),
    //         };
    //         match stdout.flush() {
    //             Ok(_) => {}
    //             Err(e) => tracing::error!("could not flush {e}"),
    //         };
    //         return Err(e.into());
    //     }
    // };

    let mut rr = Ratatui::try_new().expect("test");
    let printer = rr;

    sys_addr.do_send(start);

    // let printer = match format_clone {
    //     None | Some(OutputFormat::Normal) => Writers::Pretty,
    //     Some(OutputFormat::Json) => Writers::Json,
    // };
    // let (events_sender, mut events_receiver_2) = std::sync::mpsc::channel::<ExternalEvents>();
    // let h = thread::spawn(move || {
    //     let printer = Ratatui::try_new(events_receiver_2).expect("setup");
    // });

    let stdout = &mut std::io::stdout();

    match startup_oneshot_receiver.await? {
        Ok(DidStart::Started) => {
            let evt = StartupEvent::Started;
            match printer.handle_startup_event(stdout, &evt) {
                Ok(_) => {}
                Err(e) => tracing::error!(?e),
            };
            match stdout.flush() {
                Ok(_) => {}
                Err(e) => tracing::error!("could not flush {e}"),
            };
        }
        Err(e) => {
            let evt = StartupEvent::FailedStartup((&e).into());
            match printer.handle_startup_event(stdout, &evt) {
                Ok(_) => {}
                Err(e) => tracing::error!(?e),
            };
            match stdout.flush() {
                Ok(_) => {}
                Err(e) => tracing::error!("could not flush {e}"),
            };
            return Err(e.into());
        }
    };

    let mut rr = Ratatui::try_new().expect("test");
    let (sender, ui_handle, other) = rr.install().expect("thread install");
    let printer = sender;

    let events_handler = tokio::spawn(async move {
        // let events = vec![];
        let stdout = &mut std::io::stdout();

        while let Some(evt) = events_receiver.recv().await {
            let span = debug_span!("External Event processor");
            let _g2 = span.enter();
            tracing::debug!(external_event=?evt);
            match printer.handle_external_event(stdout, evt) {
                Ok(_v) => {}
                Err(e) => tracing::error!("could not write to stdout {e}"),
            }
            // match stdout.flush() {
            //     Ok(_) => {}
            //     Err(e) => tracing::error!("could not flush {e}"),
            // };
        }
        // events
    });

    println!("after...");

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
