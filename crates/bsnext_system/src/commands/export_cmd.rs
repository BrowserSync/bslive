use crate::args::Args;
use crate::start_kind::StartKind;
use bsnext_core::export::{export_one_server, ExportCommand, ExportEvent};
use bsnext_fs_helpers::WriteMode;
use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
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
    tracing::debug!("printer: {}", printer);

    let ctx = StartupContext::from_cwd(Some(cwd));
    tracing::debug!("StartupContext: {:?}", ctx);

    let start_kind = StartKind::from_args(args).input(&ctx);

    match start_kind {
        Err(e) => eprintln!("an error occured here?, {}", e),
        Ok(SystemStartArgs::InputOnly { input: _ }) => todo!("handle InputOnly?"),
        Ok(SystemStartArgs::PathWithInput { path: _, input }) if input.servers.len() == 1 => {
            let first = &input.servers[0];

            let write_mode = if cmd.force {
                WriteMode::Override
            } else {
                WriteMode::Safe
            };

            let events = export_one_server(cwd, first.clone(), cmd, write_mode).await?;

            let has_error = events
                .iter()
                .any(|e| matches!(e, ExportEvent::Failed { .. }));

            let stdout = &mut std::io::stdout();
            let stderr = &mut std::io::stderr();

            for export_event in &events {
                match &export_event {
                    ExportEvent::Failed { .. } => {
                        match printer.handle_export_event(stderr, export_event) {
                            Ok(_) => {}
                            Err(e) => tracing::error!(?e),
                        };
                    }
                    _ => {
                        match printer.handle_export_event(stdout, export_event) {
                            Ok(_) => {}
                            Err(e) => tracing::error!(?e),
                        };
                    }
                }
            }

            match (stderr.flush(), stdout.flush()) {
                (Ok(_), Ok(_)) => {}
                _ => tracing::error!("could not flush"),
            };
            if has_error {
                return Err(anyhow::anyhow!("export failed"));
            }
        }
        Ok(SystemStartArgs::PathWithInput { path: _, input: _ }) => {
            // let first =
            // let _result = export_one_server(cwd, cmd).await;
            todo!("handle more than 1 server for export?")
        }
        Ok(SystemStartArgs::PathWithInvalidInput { .. }) => todo!("handle PathWithInvalidInput?"),
    }
    Ok(())
}
