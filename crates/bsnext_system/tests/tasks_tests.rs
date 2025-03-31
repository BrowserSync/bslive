use actix::{Actor, Recipient, ResponseActFuture, WrapFuture};
use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal::AnyEvent;
use bsnext_dto::FilesChangedDTO;
use bsnext_system::task::{AsActor, Task, TaskCommand, TaskComms, TaskGroup, TaskGroupRunner};
use std::time::Duration;

#[actix_rt::test]
async fn test_task_group_runner() -> anyhow::Result<()> {
    let evt = AnyEvent::External(ExternalEventsDTO::FilesChanged(FilesChangedDTO {
        paths: vec!["abc.jpg".to_string()],
    }));
    let v1 = Box::new(Task::AnyEvent(evt));

    let tasks: Vec<Box<dyn AsActor>> = vec![
        mock_item(Duration::from_millis(20)),
        mock_item(Duration::from_millis(20)),
        mock_item(Duration::from_millis(20)),
        v1,
    ];
    let task_group = TaskGroup::seq(tasks);
    let task_group_runner = TaskGroupRunner::new(task_group);
    let addr = task_group_runner.start();
    let mock_server = create_mock_server();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<AnyEvent>(100);

    let r = addr
        .send(TaskCommand::Changes {
            changes: vec![],
            fs_event_context: Default::default(),
            task_comms: TaskComms {
                servers_recip: mock_server,
                any_event_sender: tx,
            },
        })
        .await;

    let evt1 = rx.recv().await;
    match evt1 {
        Some(AnyEvent::External(ExternalEventsDTO::FilesChanged(FilesChangedDTO {
            paths,
            ..
        }))) => {
            assert_eq!(vec!["abc.jpg".to_string()], paths);
        }
        _ => unreachable!("here?"),
    };
    dbg!(&r);

    Ok(())
}

fn mock_item(duration: Duration) -> Box<dyn AsActor> {
    #[derive(Debug)]
    struct F {
        pub duration: Duration,
    }
    impl Actor for F {
        type Context = actix::Context<Self>;
    }
    impl actix::Handler<TaskCommand> for F {
        type Result = ResponseActFuture<Self, ()>;

        fn handle(&mut self, msg: TaskCommand, ctx: &mut Self::Context) -> Self::Result {
            let d = self.duration;
            let a1 = async move {
                println!("will wait for {:?}", d);
                tokio::time::sleep(d).await;
            };
            Box::pin(a1.into_actor(self))
        }
    }
    impl AsActor for F {
        fn into_actor2(self: Box<Self>) -> Recipient<TaskCommand> {
            let a = self.start();
            a.recipient()
        }
    }
    let wrapper = F { duration };
    Box::new(wrapper)
}

fn create_mock_server() -> Recipient<FilesChanged> {
    struct A;
    impl Actor for A {
        type Context = actix::Context<Self>;
    }
    impl actix::Handler<FilesChanged> for A {
        type Result = ();

        fn handle(&mut self, msg: FilesChanged, ctx: &mut Self::Context) -> Self::Result {
            todo!()
        }
    }
    let s = A;
    let addr = s.start();
    addr.recipient()
}
