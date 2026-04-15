use actix::{Actor, ActorFutureExt, Recipient, ResponseActFuture, WrapFuture};
use bsnext_dto::internal::AnyEvent;
use bsnext_task::as_actor::AsActor;
use bsnext_task::invocation::Invocation;
use bsnext_task::invocation_result::InvocationResult;
use bsnext_task::task_entry::TaskEntry;
use bsnext_task::task_scope::TaskScope;
use bsnext_task::task_scope_runner::TaskScopeRunner;
use bsnext_task::task_trigger::{FsChangesTrigger, TaskTrigger, TaskTriggerSource};
use bsnext_task::{ContentId, NodePath};
use std::fmt::{Debug, Formatter};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

#[actix_rt::test]
async fn test_task_scope_runner() -> anyhow::Result<()> {
    let id1 = ContentId::new(1);
    let id2 = ContentId::new(2);
    let p1 = NodePath::root_for(id1);
    let p2 = NodePath::root_for(id2);
    let tasks: Vec<_> = vec![
        TaskEntry::new(
            mock_f(async {
                println!("did run");
                ()
            }),
            id1,
            p1,
        ),
        TaskEntry::new(
            mock_f(async {
                println!("did run 2");
                ()
            }),
            id2,
            p2,
        ),
    ];
    let task_scope = TaskScope::seq(tasks, Default::default(), 0);
    let task_scope_runner = TaskScopeRunner::new(task_scope);
    let addr = task_scope_runner.start();

    let (_tx, mut rx) = tokio::sync::mpsc::channel::<AnyEvent>(100);
    let trigger = TaskTriggerSource::FsChanges(FsChangesTrigger::new(vec![], Default::default()));
    let task_trigger = TaskTrigger::new(trigger);
    let path = NodePath::root_for(ContentId::new(0));
    let invocation = Invocation::new(&path, task_trigger);

    let task_result = addr.send(invocation).await.unwrap();
    let _evt = tokio::time::timeout(Duration::from_secs(2), rx.recv()).await;
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
        type Result = ResponseActFuture<Self, InvocationResult>;

        fn handle(&mut self, _invocation: Invocation, _ctx: &mut Self::Context) -> Self::Result {
            let f = self.f.take().unwrap();
            let p = NodePath::root_for(ContentId::new(0));
            Box::pin(f.into_actor(self).map(|_, _, _| InvocationResult::ok(p)))
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
