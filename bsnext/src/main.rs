use actix::Actor;
use std::env::current_dir;
use std::io::Write;
use std::path::PathBuf;

use bsnext_core::dto::ExternalEvents;
use bsnext_output::{OutputWriter, Writers};
use bsnext_system::args::Args;
use bsnext_system::start_kind::StartKind;
use bsnext_system::startup::{DidStart, StartupResult};
use bsnext_system::{BsSystem, Start};
use bsnext_tracing::{init_tracing, OutputFormat};
use clap::Parser;
use tokio::sync::{mpsc, oneshot};

#[actix_rt::main]
async fn main() -> Result<(), anyhow::Error> {
    std::env::set_var("RUST_LIB_BACKTRACE", "0");
    let cwd = PathBuf::from(current_dir().unwrap().to_string_lossy().to_string());
    let args = Args::parse();
    let format_clone = args.format;

    init_tracing(args.log_level, args.format);
    tracing::trace!(?args);

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

    sys_addr.do_send(start);

    match startup_oneshot_receiver.await? {
        Ok(DidStart::Started) => tracing::info!("started..."),
        Err(e) => return Err(e.into()),
    };

    let events_handler = tokio::spawn(async move {
        let events = vec![];
        let stdout = &mut std::io::stdout();
        let printer = match format_clone {
            None | Some(OutputFormat::Normal) => Writers::Pretty,
            Some(OutputFormat::Json) => Writers::Json,
        };
        while let Some(evt) = events_receiver.recv().await {
            match printer.handle_event(stdout, &evt) {
                Ok(_v) => {}
                Err(e) => tracing::error!("could not write to stdout {e}"),
            }
            match stdout.flush() {
                Ok(_) => {}
                Err(e) => tracing::error!("could not flush {e}"),
            };
            // let as_evt = ExternalEvent {
            //     level: EventLevel::External,
            //     fields: evt,
            // };
            // let json = serde_json::to_string_pretty(&as_evt).unwrap();
            // println!("{json}");
            // match handle_event(stdout, &evt) {
            //     Ok(_) => {
            //         events.push(evt);
            //     }
            //     Err(e) => tracing::error!("could not write to stdout {e}"),
            // }
        }
        events
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
        Ok(v) => {
            tracing::info!(?v, "events seen");
            let errors = v
                .iter()
                .filter(|e| matches!(e, ExternalEvents::StartupFailed(..)))
                .collect::<Vec<_>>();
            if !errors.is_empty() {
                tracing::info!("stopped for the following reasons");
                for msg in errors {
                    tracing::error!(?msg);
                }
                return Err(anyhow::anyhow!("exited..."));
            }
        }
        Err(e) => tracing::error!(?e),
    }
    Ok(())
}
