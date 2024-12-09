use crate::args::ExportCommand;
use crate::export::test_playground_export;
use std::path::PathBuf;

pub async fn export_cmd(cwd: &PathBuf, cmd: &ExportCommand) -> Result<(), anyhow::Error> {
    let result = test_playground_export(cwd, cmd).await;
    match result {
        Ok(()) => todo!("handle result"),
        Err(_) => todo!("handle error"),
    }
}
