use crate::args::{Args, SubCommands};

use crate::commands::start_command::StartCommand;
use crate::commands::{export_cmd, start_command};
use bsnext_core::shared_args::{FsOpts, InputOpts};
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
    I: IntoIterator<Item = T> + std::fmt::Debug,
    T: Into<OsString> + Clone,
{
    std::env::set_var("RUST_LIB_BACKTRACE", "0");
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
    let cmd_clone = args.command.clone();
    match cmd_clone {
        None => {
            let start_cmd = StartCommand {
                cors: false,
                port: args.port,
                trailing: args.trailing.clone(),
            };
            try_start(
                &args.fs_opts,
                &args.input_opts,
                format_clone,
                writer,
                start_cmd,
            )
            .await
        }
        Some(command) => match command {
            SubCommands::Export(cmd) => {
                let start_cmd = StartCommand {
                    cors: false,
                    port: None,
                    trailing: cmd.trailing.clone(),
                };
                let cwd = PathBuf::from(current_dir().unwrap().to_string_lossy().to_string());
                let result =
                    export_cmd::export_cmd(&cwd, &args.fs_opts, &args.input_opts, &cmd, &start_cmd)
                        .await;
                bsnext_output::stdout::completion_writer(writer, result)
            }
            SubCommands::Example(example) => {
                todo!("{:?}", example);
            }
            SubCommands::Start(start) => {
                try_start(&args.fs_opts, &args.input_opts, format_clone, writer, start).await
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

async fn try_start(
    fs_opts: &FsOpts,
    input_opts: &InputOpts,
    format: OutputFormat,
    writer: OutputWriters,
    start: StartCommand,
) -> Result<(), anyhow::Error> {
    let cwd = PathBuf::from(current_dir().unwrap().to_string_lossy().to_string());
    let (events_sender, channel_future) = stdout_channel(format);
    let fs_opts_clone = fs_opts.clone();
    let input_opts_clone = input_opts.clone();
    let start_cmd_future =
        start_command::start_cmd(cwd, fs_opts_clone, input_opts_clone, start, events_sender);
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
