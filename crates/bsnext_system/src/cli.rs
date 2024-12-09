use crate::args::{Args, SubCommands};

use crate::commands::{export_cmd, start_command};
use bsnext_tracing::{init_tracing, WriteOption};
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
    let write_opt = if args.write_log {
        WriteOption::File
    } else {
        WriteOption::None
    };

    init_tracing(args.log_level, args.format, write_opt);

    tracing::debug!("{:#?}", args);

    match &args.command {
        None => start_command::start_cmd(cwd, args).await,
        Some(command) => match command {
            SubCommands::Export(cmd) => export_cmd::export_cmd(&cwd, cmd).await,
        },
    }
}
