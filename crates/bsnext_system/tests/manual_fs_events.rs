use bsnext_core::shared_args::{FsOpts, InputOpts};
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal::AnyEvent;
use bsnext_fs::FsEvent;
use bsnext_system::start::start_command::StartCommand;
use bsnext_system::start::start_kind::StartKind;
use bsnext_system::start::start_system::start_system;
use futures_util::future::join;
use futures_util::StreamExt;
use std::fs;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

#[actix_rt::test]
async fn test_fs_events() -> Result<(), anyhow::Error> {
    let tmp_dir = tempfile::tempdir().unwrap();
    let index_file = tmp_dir.path().join("bslive.yaml");
    let input = r#"
servers:
    - name: api
      routes:
        - path: /
          html: |
            <body>
              <link href="/c.css" rel="stylesheet">
            </body>
          watch:
            run:
              - sh: echo 'oops'
        - path: /c.css
          raw: "body { background: cyan }"
    "#;
    fs::write(&index_file, input).expect("can write?");

    let cwd = PathBuf::from(tmp_dir.path());

    let (events_sender, events_receiver) = mpsc::channel::<AnyEvent>(1);
    let start = StartCommand {
        cors: false,
        port: None,
        proxies: vec![],
        trailing: vec![],
        logging: Default::default(),
        format: Default::default(),
    };
    let start_kind = StartKind::from_args(&FsOpts::default(), &InputOpts::default(), &start);
    let api = start_system(cwd, start_kind, events_sender)
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    let active_servers = api.active_servers().await?;
    let first = active_servers.first().unwrap();
    let id = first.identity.as_id();

    let r = ReceiverStream::new(events_receiver)
        // .inspect(|e| println!("{:#?}", e))
        .filter(|e| {
            futures::future::ready(matches!(
                &e,
                AnyEvent::External(ExternalEventsDTO::FileChanged(..))
            ))
        })
        .take(2)
        .collect::<Vec<_>>();

    let h1 = tokio::spawn(r);

    let h = tokio::spawn(async move {
        api.fs_event(FsEvent::changed("/c.css", "c.css", id));
        tokio::time::sleep(Duration::from_millis(10)).await;
        api.fs_event(FsEvent::changed("/c.css", "c.css", id));
    });

    let (events, _) = join(h1, h).await;
    insta::assert_debug_snapshot!(events);
    Ok(())
}
