use bsnext_core::shared_args::{FsOpts, InputOpts};
use bsnext_dto::internal::AnyEvent;
use bsnext_system::start::start_command::StartCommand;
use bsnext_system::start::start_system::start_system;
use std::fs;
use std::path::PathBuf;
use tokio::sync::mpsc;

#[actix_rt::main]
pub async fn main() -> Result<(), anyhow::Error> {
    let tmp_dir = tempfile::tempdir().unwrap();
    let index_file = tmp_dir.path().join("index.html");
    fs::write(&index_file, String::from("Hello world! (without-stdout)")).expect("can write?");

    let cwd = PathBuf::from(tmp_dir.path());
    let as_str = cwd.to_string_lossy().to_string();

    let start = StartCommand {
        cors: false,
        port: None,
        trailing: vec![as_str],
        proxies: vec![],
    };

    let (events_sender, mut events_receiver) = mpsc::channel::<AnyEvent>(1);

    let api = start_system(
        cwd,
        FsOpts::default(),
        InputOpts::default(),
        events_sender,
        start,
    )
    .await
    .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    tokio::spawn(async move {
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        if let Err(e) = api.stop().await {
            tracing::error!(?e);
        }
    });

    while let Some(evt) = events_receiver.recv().await {
        dbg!(&evt);
    }

    Ok(())
}
