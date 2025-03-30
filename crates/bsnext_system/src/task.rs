use crate::any_event_sender::AnyEventSender;
use crate::cmd::ShCmd;
use crate::tasks::notify_servers::NotifyServers;
use actix::{Actor, ActorFutureExt, Handler, Recipient, ResponseActFuture, Running, WrapFuture};
use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;
use bsnext_dto::internal::AnyEvent;
use bsnext_fs::FsEventContext;
use bsnext_input::route::{BsLiveRunner, Runner};
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
pub enum TaskCommand {
    Changes {
        changes: Vec<PathBuf>,
        fs_event_context: FsEventContext,
        task_comms: TaskComms,
    },
}

impl TaskCommand {
    pub fn comms(&self) -> &TaskComms {
        match self {
            TaskCommand::Changes { task_comms, .. } => task_comms,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskComms {
    pub any_event_sender: Sender<AnyEvent>,
    pub servers_recip: Recipient<FilesChanged>,
}

impl TaskComms {
    pub(crate) fn new(p0: Sender<AnyEvent>, p1: Recipient<FilesChanged>) -> TaskComms {
        Self {
            any_event_sender: p0,
            servers_recip: p1,
        }
    }
}

#[derive(Debug)]
pub enum Task {
    Runner(Runner),
    AnyEvent(AnyEvent),
    Group(TaskGroup),
}

impl AsActor for Task {
    fn into_actor2(self: Box<Self>) -> Recipient<TaskCommand> {
        match *self {
            Task::Runner(Runner::BsLive {
                bslive: BsLiveRunner::NotifyServer,
            }) => {
                let s = NotifyServers::new();
                let s = s.start();
                s.recipient()
            }
            Task::AnyEvent(evt) => {
                let a = AnyEventSender::new(evt);
                let a = a.start();
                a.recipient()
            }
            Task::Group(group) => {
                let group_runner = TaskGroupRunner::new(group);
                let s = group_runner.start();
                s.recipient()
            }
            Task::Runner(Runner::Sh { sh }) => {
                let cmd = ShCmd::new(sh.into());
                let s = cmd.start();
                s.recipient()
            }
            Task::Runner(Runner::ShImplicit(_)) => todo!("Task::Runner::Runner::ShImplicit"),
        }
    }
}

impl Task {
    pub fn into_actor(self) -> Recipient<TaskCommand> {
        match self {
            Task::Runner(Runner::BsLive {
                bslive: BsLiveRunner::NotifyServer,
            }) => {
                let s = NotifyServers::new();
                let s = s.start();
                s.recipient()
            }
            Task::AnyEvent(evt) => {
                let a = AnyEventSender::new(evt);
                let a = a.start();
                a.recipient()
            }
            Task::Group(group) => {
                let group_runner = TaskGroupRunner::new(group);
                let s = group_runner.start();
                s.recipient()
            }
            Task::Runner(Runner::Sh { sh }) => {
                let cmd = ShCmd::new(sh.into());
                let s = cmd.start();
                s.recipient()
            }
            Task::Runner(Runner::ShImplicit(_)) => todo!("Task::Runner::Runner::ShImplicit"),
        }
    }
}

pub trait AsActor: std::fmt::Debug {
    fn into_actor2(self: Box<Self>) -> Recipient<TaskCommand>;
}

#[derive(Debug)]
pub struct TaskGroup {
    tasks: Vec<Box<dyn AsActor>>,
}

impl TaskGroup {
    pub fn new(tasks: Vec<Box<dyn AsActor>>) -> Self {
        Self { tasks }
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }
}

pub struct TaskGroupRunner {
    done: bool,
    task_group: Option<TaskGroup>,
}

impl TaskGroupRunner {
    fn new(p0: TaskGroup) -> Self {
        Self {
            task_group: Some(p0),
            done: false,
        }
    }
}

impl actix::Actor for TaskGroupRunner {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::info!(actor.lifecycle = "started", "TaskGroupRunner2")
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::info!(" x stopped TaskGroupRunner2")
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        tracing::info!(" ⏰ stopping TaskGroupRunner2 {}", self.done);
        Running::Stop
    }
}

impl Handler<TaskCommand> for TaskGroupRunner {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: TaskCommand, _ctx: &mut Self::Context) -> Self::Result {
        tracing::debug!(done = self.done, "TaskGroupRunner::TaskCommand");
        let Some(group) = self.task_group.take() else {
            todo!("how to handle a concurrent request here?");
        };
        tracing::debug!("  └── {} tasks in group", group.len());
        let actors = group
            .tasks
            .into_iter()
            .map(|x| x.into_actor2())
            .collect::<Vec<_>>();
        let future = async move {
            for x in actors {
                match x.send(msg.clone()).await {
                    Ok(_) => tracing::trace!("did send"),
                    Err(e) => tracing::error!("{e}"),
                }
            }
        };
        // let self_addr = ctx.address();
        Box::pin(future.into_actor(self).map(|_res, actor, _ctx| {
            println!("all done!");
            actor.done = true;
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::task::{AsActor, Task, TaskCommand, TaskComms, TaskGroup, TaskGroupRunner};
    use actix::{Actor, Recipient, ResponseActFuture, WrapFuture};
    use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;
    use bsnext_dto::external_events::ExternalEventsDTO;
    use bsnext_dto::internal::AnyEvent;
    use bsnext_dto::FilesChangedDTO;
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
        let task_group = TaskGroup { tasks };
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
}
