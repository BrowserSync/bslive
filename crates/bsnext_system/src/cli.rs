use crate::args::{Args, SubCommands};

use crate::export::export_cmd;
use crate::start;
use crate::start::start_command::StartCommand;
use crate::start::stdout_channel;
use bsnext_core::shared_args::{FsOpts, InputOpts};
use bsnext_output::OutputWriters;
use bsnext_tracing::{init_tracing, OutputFormat, WriteOption};
use clap::Parser;
use std::env::current_dir;
use std::ffi::OsString;
use std::path::PathBuf;

/// The typical lifecycle when ran from a CLI environment
pub async fn from_args<I, T>(itr: I) -> Result<(), anyhow::Error>
where
    I: IntoIterator<Item = T> + std::fmt::Debug,
    T: Into<OsString> + Clone,
{
    std::env::set_var("RUST_LIB_BACKTRACE", "0");
    let args = Args::parse_from(itr);
    let cwd = PathBuf::from(current_dir().unwrap().to_string_lossy().to_string());

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
    let command = args.command.unwrap_or_else(|| {
        SubCommands::Start(StartCommand {
            cors: false,
            port: args.port,
            trailing: args.trailing.clone(),
            proxies: vec![],
        })
    });

    match command {
        SubCommands::Export(cmd) => {
            let start_cmd = StartCommand {
                cors: false,
                port: None,
                trailing: cmd.trailing.clone(),
                proxies: vec![],
            };
            let cwd = PathBuf::from(current_dir().unwrap().to_string_lossy().to_string());
            let result = export_cmd(&cwd, &args.fs_opts, &args.input_opts, &cmd, &start_cmd).await;
            bsnext_output::stdout::completion_writer(writer, result)
        }
        SubCommands::Example(example) => {
            todo!("{:?}", example);
        }
        SubCommands::Start(start) => {
            stdout_wrapper(start, args.fs_opts, args.input_opts, cwd, writer).await
        }
    }
}

async fn stdout_wrapper(
    start_cmd: StartCommand,
    fs_opts: FsOpts,
    input_opts: InputOpts,
    cwd: PathBuf,
    writer: OutputWriters,
) -> anyhow::Result<()> {
    let (events_sender, channel_future) = stdout_channel(writer);
    let system_handle = actix_rt::spawn(start::with_sender(
        cwd,
        fs_opts,
        input_opts,
        events_sender,
        start_cmd,
    ));
    let channel_handle = actix_rt::spawn(channel_future);
    let output = tokio::select! {
        r = system_handle => {
            match r {
                Ok(Ok(..)) => Ok(()),
                Ok(Err(err)) => Err(anyhow::anyhow!("1{}", err)),
                Err(e) => Err(anyhow::anyhow!("2{}", e))
            }
        }
        r = channel_handle => {
            match r {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow::anyhow!("3{}", e))
            }
        }
    };
    output
}
