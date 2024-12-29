use crate::args::Args;
use crate::commands::start_command::StartCommand;
use crate::start_kind::StartKind;
use bsnext_core::export::{export_one_server, ExportCommand};
use bsnext_fs_helpers::WriteMode;
use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
use bsnext_output::OutputWriterTrait;
use std::path::PathBuf;

pub async fn export_cmd(
    cwd: &PathBuf,
    args: &Args,
    cmd: &ExportCommand,
    start_command: &StartCommand,
) -> Result<Vec<impl OutputWriterTrait>, impl OutputWriterTrait> {
    let ctx = StartupContext::from_cwd(Some(cwd));
    tracing::debug!("StartupContext: {:?}", ctx);

    let start_kind = StartKind::from_args(args, start_command).input(&ctx);

    match start_kind {
        Err(e) => todo!(
            "handle errors here, eg: when the startup was invalid {:?}",
            e
        ),
        Ok(SystemStartArgs::InputOnly { input: _ }) => todo!("handle InputOnly?"),
        Ok(SystemStartArgs::PathWithInput { path: _, input }) if input.servers.len() == 1 => {
            let first = &input.servers[0];

            let fs_write_mode = if args.fs_opts.force {
                WriteMode::Override
            } else {
                WriteMode::Safe
            };

            export_one_server(cwd, first.clone(), cmd, fs_write_mode).await
        }
        Ok(SystemStartArgs::PathWithInput { path: _, input: _ }) => {
            // let first =
            // let _result = export_one_server(cwd, cmd).await;
            todo!("handle more than 1 server for export?")
        }
        Ok(SystemStartArgs::PathWithInvalidInput { .. }) => todo!("handle PathWithInvalidInput?"),
    }
}
