use bsnext_core::shared_args::{FsOpts, InputOpts};
use bsnext_output::OutputWriters;
use bsnext_system::start;
use bsnext_system::start::start_command::StartCommand;
use std::fs;
use std::path::PathBuf;

// todo: add the higher-level tests like these
pub async fn test_starting_with_stdout() -> Result<(), anyhow::Error> {
    let tmp_dir = tempfile::tempdir().unwrap();
    let index_file = tmp_dir.path().join("index.html");
    fs::write(&index_file, String::from("helloworld")).expect("can write?");

    let pb = PathBuf::from(tmp_dir.path());
    let as_str = pb.to_string_lossy().to_string();

    let start = StartCommand {
        cors: false,
        port: None,
        trailing: vec![as_str],
    };

    start::with_stdout(
        pb,
        &FsOpts::default(),
        &InputOpts::default(),
        OutputWriters::default(),
        start,
    )
    .await
}
