use crate::any_event_sender::AnyEventSender;
use crate::cmd::ShCmd;
use crate::runner::{RunKind, Runnable, Runner};
use actix::{Actor, ActorFutureExt, Handler, Recipient, ResponseActFuture, Running, WrapFuture};
use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;
use bsnext_dto::internal::AnyEvent;
use bsnext_fs::FsEventContext;
use bsnext_input::route::BsLiveRunner;
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
    Runnable(Runnable),
    AnyEvent(AnyEvent),
    Group(TaskGroup),
}

impl AsActor for Task {
    fn into_actor2(self: Box<Self>) -> Recipient<TaskCommand> {
        match *self {
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
            Task::Runnable(runnable) => Box::new(runnable).into_actor2(),
        }
    }
}

impl Task {
    pub fn into_actor(self) -> Recipient<TaskCommand> {
        Box::new(self).into_actor2()
    }
}

pub trait AsActor: std::fmt::Debug {
    fn into_actor2(self: Box<Self>) -> Recipient<TaskCommand>;
}

#[derive(Debug)]
pub struct TaskGroup {
    run_kind: RunKind,
    tasks: Vec<Box<dyn AsActor>>,
}

impl From<Runner> for TaskGroup {
    fn from(value: Runner) -> Self {
        let boxed_tasks = value
            .tasks
            .into_iter()
            .map(|x| -> Box<dyn AsActor> { Box::new(x) })
            .collect::<Vec<Box<dyn AsActor>>>();
        match value.kind {
            RunKind::Sequence => Self::seq(boxed_tasks),
            RunKind::Overlapping => Self::seq(boxed_tasks),
        }
    }
}

impl From<&Runner> for TaskGroup {
    fn from(value: &Runner) -> Self {
        TaskGroup::from(value.clone())
    }
}

impl TaskGroup {
    pub fn seq(tasks: Vec<Box<dyn AsActor>>) -> Self {
        Self {
            run_kind: RunKind::Sequence,
            tasks,
        }
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
    pub fn new(p0: TaskGroup) -> Self {
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
            actor.done = true;
        }))
    }
}
