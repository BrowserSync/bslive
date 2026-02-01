use actix::{Actor, ActorFutureExt, Recipient, ResponseActFuture, WrapFuture};
use bsnext_dto::internal::{AnyEvent, InvocationId, TaskResult};
use bsnext_task::as_actor::AsActor;
use bsnext_task::invocation::Invocation;
use bsnext_task::task_entry::TaskEntry;
use bsnext_task::task_group::TaskGroup;
use bsnext_task::task_group_runner::TaskGroupRunner;
use bsnext_task::task_trigger::{TaskComms, TaskTrigger, TaskTriggerSource};
use std::any::Any;
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::pin::Pin;

#[actix_rt::test]
async fn test_task_group_runner() -> anyhow::Result<()> {
    let tasks: Vec<_> = vec![
        TaskEntry::new(
            mock_f(async {
                println!("did run");
                ()
            }),
            1,
        ),
        TaskEntry::new(
            mock_f(async {
                println!("did run 2");
                ()
            }),
            2,
        ),
    ];
    let task_group = TaskGroup::seq(tasks, Default::default(), 0);
    let task_group_runner = TaskGroupRunner::new(task_group);
    let addr = task_group_runner.start();

    let (tx, mut rx) = tokio::sync::mpsc::channel::<AnyEvent>(100);
    let trigger = TaskTrigger {
        variant: TaskTriggerSource::FsChanges {
            changes: vec![],
            fs_event_context: Default::default(),
        },
        comms: TaskComms {
            any_event_sender: tx,
        },
        invocation_id: 0,
    };
    let id = 0;
    let one_task = Invocation(0, trigger);

    let task_result = addr.send(one_task).await.unwrap();
    let _evt = rx.recv().await;
    assert_eq!(task_result.task_reports.len(), 2);
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
    impl actix::Handler<Invocation> for A {
        type Result = ResponseActFuture<Self, TaskResult>;

        fn handle(
            &mut self,
            Invocation(id, trigger): Invocation,
            _ctx: &mut Self::Context,
        ) -> Self::Result {
            let f = self.f.take().unwrap();
            Box::pin(
                f.into_actor(self)
                    .map(|_, _, _| TaskResult::ok(InvocationId(0))),
            )
        }
    }
    impl AsActor for A {
        fn into_task_recipient(self: Box<Self>) -> Recipient<Invocation> {
            let a = self.start();
            a.recipient()
        }
    }
    let wrapper = A {
        f: Some(Box::pin(f)),
    };
    Box::new(wrapper)
}
