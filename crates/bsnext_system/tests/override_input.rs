use actix::Actor;
use bsnext_dto::internal::{AnyEvent, InternalEvents};
use bsnext_input::startup::{DidStart, StartupResult};
use bsnext_input::Input;
use bsnext_system::monitor::OverrideInput;
use bsnext_system::start_kind::StartKind;
use bsnext_system::{BsSystem, Start};
use std::env::current_dir;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

#[actix_rt::test]
pub async fn main() -> Result<(), anyhow::Error> {
    let input = r#"
servers:
    - name: api
      routes:
        - path: /
          html: abc
    "#;
    let input = Input::from_str(input).expect("ay?");
    let input2 = r#"
servers:
    - name: api
      routes:
        - path: /
          html: def
    "#;
    let input2 = Input::from_str(input2).expect("ay?");
    let ids: Vec<u64> = input.servers.iter().map(|x| x.identity.as_id()).collect();
    let id = ids.get(0).expect("first").to_owned();
    let start_kind = StartKind::from_input(input);

    // this will be something like `/Users/shaneosbourne/WebstormProjects/bslive`
    //                         or  `/Users/shaneosbourne/WebstormProjects/bslive/crates/bsnext_system`
    let cwd = PathBuf::from(current_dir().unwrap().to_string_lossy().to_string());
    dbg!(&cwd);

    let (tx, rx) = oneshot::channel();
    let (startup_oneshot_sender, startup_oneshot_receiver) = oneshot::channel::<StartupResult>();
    let (events_sender, events_receiver) = mpsc::channel::<AnyEvent>(1);

    let start = Start {
        kind: start_kind,
        cwd: Some(cwd),
        ack: tx,
        events_sender,
        startup_oneshot_sender,
    };

    let system = BsSystem::new();
    let sys_addr = system.start();
    let sys_clone = sys_addr.clone();

    sys_addr.do_send(start);

    match startup_oneshot_receiver.await {
        Ok(Ok(DidStart::Started)) => {
            // after 100ms, send an override for an input
            tokio::spawn(async move {
                tokio::time::sleep(Duration::from_millis(100)).await;
                sys_clone.do_send(OverrideInput { input: input2 });
            });
            // let mut count = 0;
            let stream = Box::pin(
                ReceiverStream::new(events_receiver)
                    .take(2)
                    .collect::<Vec<_>>(),
            );
            let evts = stream.await;
            let first = evts.get(0).unwrap();
            let second = evts.get(1).unwrap();
            assert!(matches!(
                first,
                AnyEvent::Internal(InternalEvents::ServersChanged { .. })
            ));
            assert!(matches!(
                second,
                AnyEvent::Internal(InternalEvents::ServersChanged { .. })
            ));
        }
        Ok(Err(_)) => {}
        Err(e) => {
            eprintln!("{e}")
        }
    };
    Ok(())
}
