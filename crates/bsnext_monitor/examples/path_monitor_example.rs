use actix::{Actor, ResponseFuture};
use actix_rt::System;
use bsnext_fs::{FsEvent, FsEventContext, FsEventGrouping};
use bsnext_input::route::{DebounceDuration, Spec};
use bsnext_monitor::path_monitor::PathMonitor;
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
    System::current().stop_with_code(code);
    process::exit(code)
}

async fn async_main() -> i32 {
    struct Consumer;

    #[derive(actix::Message, Debug, Clone)]
    #[rtype(result = "()")]
    struct Ping;

    impl actix::Actor for Consumer {
        type Context = actix::Context<Self>;
    }

    impl actix::Handler<FsEventGrouping> for Consumer {
        type Result = ();

        fn handle(&mut self, _msg: FsEventGrouping, _ctx: &mut Self::Context) -> Self::Result {
            dbg!("got <FsEventGrouping>");
        }
    }

    impl actix::Handler<Ping> for Consumer {
        type Result = ResponseFuture<()>;

        fn handle(&mut self, _msg: Ping, _ctx: &mut Self::Context) -> Self::Result {
            Box::pin(async {})
        }
    }

    let consumer = Consumer;
    let actor = consumer.start();
    let reciever = actor.clone().recipient();
    let cwd = current_dir().unwrap();
    let fs_context = FsEventContext::default();
    let spec = Spec {
        debounce: Some(DebounceDuration::Ms(0)),
        ..Default::default()
    };

    let p = vec![];

    let monitor = PathMonitor::new(reciever, Default::default(), cwd, fs_context, spec, p);
    let monitor_actor = monitor.start();
    let _did_wait = actor.send(Ping).await;
    let _sent = monitor_actor
        .send(FsEvent::changed("/a", "a", fs_context.id()))
        .await;
    tokio::time::sleep(Duration::from_millis(310)).await;
    0
}
