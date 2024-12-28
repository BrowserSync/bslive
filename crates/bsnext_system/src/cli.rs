use crate::args::{Args, SubCommands};

use crate::commands::{export_cmd, start_command};
use bsnext_output::OutputWriters;
use bsnext_tracing::{init_tracing, OutputFormat, WriteOption};
use clap::Parser;
use std::env::current_dir;
use std::ffi::OsString;
use std::path::PathBuf;

/// The typical lifecycle when ran from a CLI environment
pub async fn from_args<I, T>(itr: I) -> Result<(), anyhow::Error>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + Clone,
{
    std::env::set_var("RUST_LIB_BACKTRACE", "0");
    let cwd = PathBuf::from(current_dir().unwrap().to_string_lossy().to_string());
    let args = Args::parse_from(itr);

    let write_log_opt = if args.write_log {
        WriteOption::File
    } else {
        WriteOption::None
    };

    init_tracing(args.log_level, args.format, write_log_opt);

    tracing::debug!("{:#?}", args);

    let format_clone = args.format;

    let writer = match format_clone {
        OutputFormat::Tui => OutputWriters::Pretty,
        OutputFormat::Normal => OutputWriters::Pretty,
        OutputFormat::Json => OutputWriters::Json,
    };

    tracing::debug!("printer: {}", writer);

    match &args.command {
        None => {
            let result = start_command::start_cmd(cwd, args).await;
            match result {
                Ok(_) => {
                    // noop
                    Ok(())
                }
                Err(err) => bsnext_output::stdout::write_one_err(writer, err),
            }
        }
        Some(command) => match command {
            SubCommands::Export(cmd) => {
                let result = export_cmd::export_cmd(&cwd, cmd, &args).await;
                bsnext_output::stdout::completion_writer(writer, result)
            }
        },
    }
}
