use crate::args::Args;
use crate::start_kind::start_from_inputs::StartFromInputPaths;
use crate::start_kind::StartKind;
use bsnext_core::export::{export_one_server, ExportCommand};
use bsnext_fs_helpers::WriteMode;
use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
use bsnext_input::InputError;
use bsnext_output::{OutputWriter, Writers};
use bsnext_tracing::OutputFormat;
use std::io::Write;
use std::path::PathBuf;

pub async fn export_cmd(
    cwd: &PathBuf,
    cmd: &ExportCommand,
    args: &Args,
) -> Result<(), anyhow::Error> {
    let format_clone = args.format;
    let printer = match format_clone {
        OutputFormat::Tui => Writers::Pretty,
        OutputFormat::Json => Writers::Json,
        OutputFormat::Normal => Writers::Pretty,
    };
    let ctx = StartupContext::from_cwd(Some(cwd));
    let start_kind = StartKind::from_args(args).input(&ctx);

    match start_kind {
        Err(e) => eprintln!("an error occured here"),
        Ok(SystemStartArgs::InputOnly { input }) => todo!("handle InputOnly"),
        Ok(SystemStartArgs::PathWithInput { path, input }) if input.servers.len() == 1 => {
            let first = &input.servers[0];
            let write_mode = if args.force {
                WriteMode::Override
            } else {
                WriteMode::Safe
            };
            let result = export_one_server(cwd, first.clone(), cmd, write_mode).await?;
            dbg!(&result);
            let stdout = &mut std::io::stdout();
            for export_event in result {
                // printer.handle_export_event(stdout, &export_event)?;
                match printer.handle_export_event(stdout, &export_event) {
                    Ok(_) => {}
                    Err(e) => tracing::error!(?e),
                };
            }
            match stdout.flush() {
                Ok(_) => {}
                Err(e) => tracing::error!("could not flush {e}"),
            };
        }
        Ok(SystemStartArgs::PathWithInput { path, input }) => {
            // let first =
            // let _result = export_one_server(cwd, cmd).await;
            todo!("handle ion")
        }
        Ok(SystemStartArgs::PathWithInvalidInput { .. }) => todo!("handle PathWithInvalidInput"),
    }
    //
    // // for the startup message, don't allow a TUI yet

    // todo!("handle the output from the export command");
    // match result {
    //     Ok(()) => todo!("handle result"),
    //     Err(_) => todo!("handle error"),
    // }
    Ok(())
}
