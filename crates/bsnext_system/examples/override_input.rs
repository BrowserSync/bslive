use bsnext_dto::any_event::AnyEvent;
use bsnext_system::start::start_kind::start_from_inputs::StartFromInputPaths;
use bsnext_system::start::start_kind::StartKind;
use bsnext_system::start::start_system::start_system;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;

#[actix_rt::main]
pub async fn main() -> Result<(), anyhow::Error> {
    let input = r#"
servers:
    - name: api
      routes:
        - path: /
          html: abc
    "#;
    let input2 = r#"
servers:
    - name: api
      routes:
        - path: /
          html: def
    "#;

    let tmp_dir = tempfile::tempdir().unwrap();
    let index_file = tmp_dir.path().join("bslive.yaml");
    let index_file_str = index_file.to_string_lossy().to_string();
    fs::write(&index_file, input).expect("can write?");

    let cwd = PathBuf::from(tmp_dir.path());

    let start = StartFromInputPaths {
        input_paths: vec![index_file_str],
        port: None,
        no_watch: false,
    };

    let (events_sender, _) = mpsc::channel::<AnyEvent>(1);

    let start_kind = StartKind::FromInputPaths(start);
    let api = start_system(cwd, start_kind, events_sender)
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    let Some(api) = api else {
        unreachable!("failed if we get here")
    };

    let active_servers = api.active_servers().await?;
    let s1 = active_servers.first().unwrap();

    let url = format!("http://{}", s1.socket_addr);
    let res = reqwest::get(&url).await?;
    let body = res.text().await?;
    println!("body={}", body);
    fs::write(&index_file, input2).expect("can write again?");
    tokio::time::sleep(Duration::from_millis(500)).await;
    let url = format!("http://{}", s1.socket_addr);
    let res = reqwest::get(&url).await?;
    let body = res.text().await?;
    println!("body2={}", body);

    Ok(())
}
