use actix::{Actor, ActorFutureExt, Recipient, ResponseActFuture, WrapFuture};
use bsnext_dto::internal::AnyEvent;
use bsnext_system::task::{AsActor, TaskCommand, TaskComms, TaskResult};
use bsnext_system::task_group::TaskGroup;
use bsnext_system::task_group_runner::TaskGroupRunner;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::pin::Pin;

#[actix_rt::test]
async fn test_task_group_runner() -> anyhow::Result<()> {
    // let evt = AnyEvent::External(ExternalEventsDTO::FilesChanged(FilesChangedDTO {
    //     paths: vec!["abc.jpg".to_string()],
    // }));
    // let v1 = Box::new(Task::AnyEvent(evt));

    let tasks: Vec<_> = vec![
        // mock_item(Duration::from_millis(20)),
        // mock_item(Duration::from_millis(20)),
        // mock_item(Duration::from_millis(20)),
        mock_f(async {
            println!("did run");
            ()
        }),
        mock_f(async {
            println!("did run 2");
            ()
        }),
        // v1,
    ];
    let task_group = TaskGroup::seq(tasks);
    let task_group_runner = TaskGroupRunner::new(task_group);
    let addr = task_group_runner.start();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<AnyEvent>(100);

    let r = addr
        .send(TaskCommand::Changes {
            changes: vec![],
            fs_event_context: Default::default(),
            task_comms: TaskComms {
                servers_recip: None,
                any_event_sender: tx,
            },
            invocation_id: 0,
        })
        .await;
    let evt1 = rx.recv().await;
    dbg!(&evt1);
    dbg!(&r);
    Ok(())
}

fn mock_f(f: impl Future<Output = ()> + 'static) -> Box<dyn AsActor> {
    struct A {
        f: Option<Pin<Box<dyn Future<Output = ()>>>>,
    }
    impl Debug for A {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("mock_f: A").finish()
        }
    }

    impl Actor for A {
        type Context = actix::Context<Self>;
    }
    impl actix::Handler<TaskCommand> for A {
        type Result = ResponseActFuture<Self, TaskResult>;

        fn handle(&mut self, _msg: TaskCommand, _ctx: &mut Self::Context) -> Self::Result {
            let f = self.f.take().unwrap();
            Box::pin(f.into_actor(self).map(|_, _, _| TaskResult::ok(0)))
        }
    }
    impl AsActor for A {
        fn into_actor2(self: Box<Self>) -> Recipient<TaskCommand> {
            let a = self.start();
            a.recipient()
        }
    }
    let wrapper = A {
        f: Some(Box::pin(f)),
    };
    Box::new(wrapper)
}
