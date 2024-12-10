use crate::args::Args;
use bsnext_core::export::{test_playground_export, ExportCommand};
use bsnext_output::Writers;
use bsnext_tracing::OutputFormat;
use std::path::PathBuf;

pub async fn export_cmd(
    cwd: &PathBuf,
    cmd: &ExportCommand,
    args: &Args,
) -> Result<(), anyhow::Error> {
    let format_clone = args.format;
    let _result = test_playground_export(cwd, cmd).await;

    // for the startup message, don't allow a TUI yet
    let _start_printer = match format_clone {
        OutputFormat::Tui => Writers::Pretty,
        OutputFormat::Json => Writers::Json,
        OutputFormat::Normal => Writers::Pretty,
    };

    todo!("handle the output from the export command");
    // match result {
    //     Ok(()) => todo!("handle result"),
    //     Err(_) => todo!("handle error"),
    // }
}
