use actix::{Actor, ResponseFuture};
use actix_rt::System;
use bsnext_fs::{FsEvent, FsEventContext};
use bsnext_input::route::{DebounceDuration, WatchSpec};
use bsnext_monitor::FsGroup;
use bsnext_monitor::path_monitor::{PathMonitor, WatchPaths};
use std::env::current_dir;
use std::process;
use std::time::Duration;

fn main() {
    let code = System::with_tokio_rt(|| {
        // build system with a multi-thread tokio runtime.
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap()
    })
    .block_on(async_main());
    match code {
        Ok(code) => {
            System::current().stop_with_code(code);
            process::exit(code)
        }
        Err(err) => {
            eprintln!("{err}");
            System::current().stop_with_code(1);
            process::exit(1)
        }
    }
}

async fn async_main() -> anyhow::Result<i32> {
    #[derive(Default)]
    struct Consumer {
        events: Vec<FsGroup>,
    }

    #[derive(actix::Message, Debug, Clone)]
    #[rtype(result = "()")]
    struct Ping;

    impl actix::Actor for Consumer {
        type Context = actix::Context<Self>;
    }

    impl actix::Handler<FsGroup> for Consumer {
        type Result = ();

        fn handle(&mut self, msg: FsGroup, _ctx: &mut Self::Context) -> Self::Result {
            self.events.push(msg);
        }
    }

    impl actix::Handler<Ping> for Consumer {
        type Result = ResponseFuture<()>;

        fn handle(&mut self, _msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
            Box::pin(async {})
        }
    }

    #[derive(actix::Message, Debug, Clone)]
    #[rtype(result = "Vec<FsGroup>")]
    struct Read;

    impl actix::Handler<Read> for Consumer {
        type Result = Vec<FsGroup>;

        fn handle(&mut self, _msg: Read, _ctx: &mut Self::Context) -> Self::Result {
            self.events.clone()
        }
    }

    let consumer = Consumer::default();
    let actor = consumer.start();
    let reciever = actor.clone().recipient();
    let cwd = current_dir().unwrap();
    let fs_context = FsEventContext::default();
    let debounce = bsnext_fs::Debounce::Buffered {
        duration: Duration::from_millis(0),
    };
    let spec = WatchSpec {
        debounce: Some(DebounceDuration::Ms(0)),
        ..Default::default()
    };

    let monitor = PathMonitor::new(reciever, debounce, cwd, fs_context, spec);
    let monitor_actor = monitor.start();
    monitor_actor.send(WatchPaths { paths: vec![] }).await?;
    let _did_wait = actor.send(Ping).await;
    let _sent = monitor_actor
        .send(FsEvent::changed("/a", "a", fs_context.id()))
        .await;
    let r = Read;
    tokio::time::sleep(Duration::from_millis(10)).await;
    let events = actor.send(r).await?;
    assert_eq!(events.len(), 1);
    Ok(0)
}
