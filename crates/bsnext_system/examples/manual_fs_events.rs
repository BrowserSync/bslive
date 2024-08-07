use actix::Actor;
use bsnext_dto::internal::{AnyEvent, InternalEvents};
use bsnext_fs::{ChangeEvent, FsEvent, FsEventContext, FsEventKind};
use bsnext_input::startup::DidStart;
use bsnext_input::Input;
use bsnext_system::start_kind::StartKind;
use bsnext_system::{BsSystem, Start};
use std::env::current_dir;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};

#[actix_rt::main]
pub async fn main() -> Result<(), anyhow::Error> {
    let input = r#"
servers:
    - name: api
      routes:
        - path: /
          html: |
            <body>
              <link href="/c.css" rel="stylesheet">
            </body>
        - path: /c.css
          raw: "body { background: cyan }"
    "#;
    let input = Input::from_str(input).expect("ay?");
    let ids: Vec<u64> = input.servers.iter().map(|x| x.identity.as_id()).collect();
    let id = ids.get(0).expect("first").to_owned();
    let start_kind = StartKind::from_input(input);

    // this will be something like `/Users/shaneosbourne/WebstormProjects/bslive`
    //                         or  `/Users/shaneosbourne/WebstormProjects/bslive/crates/bsnext_system`
    let cwd = PathBuf::from(current_dir().unwrap().to_string_lossy().to_string());
    dbg!(&cwd);

    let (tx, _rx) = oneshot::channel();
    let (events_sender, mut events_receiver) = mpsc::channel::<AnyEvent>(1);

    let start = Start {
        kind: start_kind,
        cwd: Some(cwd),
        ack: tx,
        events_sender,
    };

    let system = BsSystem::new();
    let sys_addr = system.start();
    let sys_clone = sys_addr.clone();

    match sys_addr.send(start).await {
        Ok(Ok(DidStart::Started)) => {
            tokio::spawn(async move {
                loop {
                    sys_clone.do_send(FsEvent {
                        kind: FsEventKind::Change(ChangeEvent {
                            absolute_path: PathBuf::from("/c.css"),
                            path: PathBuf::from("c.css"),
                        }),
                        ctx: FsEventContext { id },
                        span: None,
                    });
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }
            });
            while let Some(evt) = events_receiver.recv().await {
                match evt {
                    AnyEvent::Internal(InternalEvents::ServersChanged {
                        server_resp,
                        child_results: _,
                    }) => {
                        dbg!(server_resp);
                    }
                    AnyEvent::External(_) => {}
                }
            }
        }
        Ok(Err(_)) => {}
        Err(e) => {
            eprintln!("{e}")
        }
    };
    Ok(())
}
