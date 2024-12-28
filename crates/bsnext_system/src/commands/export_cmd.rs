use crate::args::Args;
use crate::start_kind::StartKind;
use bsnext_core::export::{export_one_server, ExportCommand};
use bsnext_fs_helpers::WriteMode;
use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
use bsnext_output::{OutputWriter, Writers};
use bsnext_output2::{OutputWriters, StdoutTarget};
use bsnext_tracing::OutputFormat;
use std::path::PathBuf;

pub async fn export_cmd(
    cwd: &PathBuf,
    cmd: &ExportCommand,
    args: &Args,
) -> Result<(), anyhow::Error> {
    let format_clone = args.format;

    let printer = match format_clone {
        OutputFormat::Tui => OutputWriters::Pretty,
        OutputFormat::Normal => OutputWriters::Pretty,
        OutputFormat::Json => OutputWriters::Json,
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

            let results = export_one_server(cwd, first.clone(), cmd, write_mode).await;

            let stdout = &mut std::io::stdout();
            let stderr = &mut std::io::stderr();
            let mut writer = StdoutTarget::new(stdout, stderr);

            return match results {
                Ok(events) => {
                    for export_event in &events {
                        printer.write_evt(export_event, &mut writer.output())?;
                    }
                    writer.close();
                    Ok(())
                }
                Err(err) => {
                    printer.write_evt(err, &mut writer.error())?;
                    writer.close();
                    Err(anyhow::anyhow!("export failed"))
                }
            };
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
