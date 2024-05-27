#![deny(clippy::all)]

#[macro_use]
extern crate napi_derive;

use actix::Actor;
use std::env::current_dir;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use clap::Parser;

use bsnext_dto::ExternalEvents;
use bsnext_output::{OutputWriter, Writers};
use bsnext_system::args::Args;
use bsnext_system::start_kind::StartKind;
use bsnext_system::startup::{DidStart, StartupResult};
use bsnext_system::{BsSystem, Start};
use bsnext_tracing::{init_tracing, OutputFormat, WriteOption};
use tokio::sync::{mpsc, oneshot};
use tokio::time::sleep;

#[napi]
async fn start(_args: Vec<String>) -> napi::bindgen_prelude::Result<i32> {
    eprintln!("async not supported yet");
    sleep(Duration::from_secs(2)).await;
    Ok(32)
}

/// Launch in a blocking way
#[napi]
fn start_sync(args: Vec<String>) -> napi::bindgen_prelude::Result<i32> {
    let sys = actix_rt::System::new();
    println!("sync args {:?}", args);
    std::env::set_var("RUST_LIB_BACKTRACE", "0");
    let result = sys.block_on(async move {
        match main_sync(args).await {
            Ok(_) => 0,
            Err(e) => {
                eprintln!("{:?}", e);
                1
            }
        }
    });
    Ok(result)
}

async fn main_sync(args: Vec<String>) -> Result<(), anyhow::Error> {
    let cwd = PathBuf::from(current_dir().unwrap().to_string_lossy().to_string());
    let args = Args::parse_from(&args);
    let format_clone = args.format;

    let write_opt = if args.write_log {
        WriteOption::File
    } else {
        WriteOption::None
    };
    init_tracing(args.log_level, args.format, write_opt);
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

    sys_addr.do_send(start);

    match startup_oneshot_receiver.await {
        Ok(Ok(DidStart::Started)) => tracing::info!("started..."),
        Ok(Err(e)) => return Err(e.into()),
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
