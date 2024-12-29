use crate::args::{Args, SubCommands};

use crate::commands::start_command::StartCommand;
use crate::commands::{export_cmd, start_command};
use bsnext_dto::internal::AnyEvent;
use bsnext_output::stdout::StdoutTarget;
use bsnext_output::OutputWriters;
use bsnext_tracing::{init_tracing, OutputFormat, WriteOption};
use clap::Parser;
use std::env::current_dir;
use std::ffi::OsString;
use std::future::Future;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
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

    let write_log_opt = if args.logging.write_log {
        WriteOption::File
    } else {
        WriteOption::None
    };

    init_tracing(args.logging.log_level, args.format, write_log_opt);

    tracing::debug!("{:#?}", args);

    let format_clone = args.format;

    let writer = match format_clone {
        OutputFormat::Tui => OutputWriters::Pretty,
        OutputFormat::Normal => OutputWriters::Pretty,
        OutputFormat::Json => OutputWriters::Json,
    };
    tracing::debug!("writer: {}", writer);

    // create a channel onto which commands can publish events
    let (events_sender, channel_future) = stdout_channel(format_clone);
    let cmd_clone = args.command.clone();
    match cmd_clone {
        None => todo!("unreachable?"),
        Some(command) => match command {
            SubCommands::Export(cmd) => {
                let start_cmd = StartCommand {
                    cors: false,
                    port: None,
                    paths: cmd.paths.clone(),
                };
                let result = export_cmd::export_cmd(&cwd, &args, &cmd, start_cmd).await;
                bsnext_output::stdout::completion_writer(writer, result)
            }
            SubCommands::Example(example) => {
                dbg!(&example);
                Ok(())
            }
            SubCommands::Start(start) => {
                let arg_clone = args.clone();
                let start_cmd_future =
                    start_command::start_cmd(cwd, arg_clone, start, events_sender);
                tokio::select! {
                    r = actix_rt::spawn(start_cmd_future) => {
                        match r {
                            Ok(Ok(_)) => Ok(()),
                            Ok(Err(err)) => bsnext_output::stdout::write_one_err(writer, err),
                            Err(er) => Err(anyhow::anyhow!("{:?}", er)),
                        }
                    }
                    h = actix_rt::spawn(channel_future) => {
                        match h {
                            Ok(_) => Ok(()),
                            Err(er) => Err(anyhow::anyhow!("{:?}", er))
                        }
                    }
                }
            }
        },
    }
}

fn stdout_channel(format: OutputFormat) -> (Sender<AnyEvent>, impl Future<Output = ()>) {
    let (events_sender, mut events_receiver) = mpsc::channel::<AnyEvent>(1);
    let channel_future = async move {
        let printer = match format {
            OutputFormat::Tui => todo!("re-implement ratatui"),
            OutputFormat::Json => OutputWriters::Json,
            OutputFormat::Normal => OutputWriters::Pretty,
        };
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
    };
    (events_sender, channel_future)
}
