use crate::runner::{RunKind, Runnable, Runner};
use actix::{Actor, ActorFutureExt, Handler, Recipient, ResponseActFuture, Running, WrapFuture};
use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;
use bsnext_dto::internal::AnyEvent;
use bsnext_fs::FsEventContext;
use futures_util::future::FutureExt;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use tokio::task::JoinSet;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "TaskResult")]
pub enum TaskCommand {
    Changes {
        changes: Vec<PathBuf>,
        fs_event_context: FsEventContext,
        task_comms: TaskComms,
        invocation_id: u64,
    },
}

impl TaskCommand {
    pub fn comms(&self) -> &TaskComms {
        match self {
            TaskCommand::Changes { task_comms, .. } => task_comms,
        }
    }
    pub fn id(&self) -> u64 {
        match self {
            TaskCommand::Changes { invocation_id, .. } => *invocation_id,
        }
    }
}

#[derive(Debug)]
pub struct TaskResult {
    #[allow(dead_code)]
    status: TaskStatus,
    #[allow(dead_code)]
    invocation_id: u64,
    #[allow(dead_code)]
    task_results: Vec<TaskResult>,
}

impl TaskResult {
    pub fn ok(id: u64) -> Self {
        Self {
            status: TaskStatus::Ok(TaskOk),
            invocation_id: id,
            task_results: vec![],
        }
    }
    pub fn ok_tasks(id: u64, tasks: Vec<TaskResult>) -> Self {
        Self {
            status: TaskStatus::Ok(TaskOk),
            invocation_id: id,
            task_results: tasks,
        }
    }
}

#[derive(Debug)]
pub enum TaskStatus {
    Ok(TaskOk),
    Err(TaskError),
}

#[derive(Debug)]
pub struct TaskOk;

#[derive(Debug, thiserror::Error)]
pub enum TaskError {
    #[error("lol")]
    FailedMsg(String),
}

#[derive(Debug, Clone)]
pub struct TaskComms {
    pub any_event_sender: Sender<AnyEvent>,
    pub servers_recip: Option<Recipient<FilesChanged>>,
}

impl TaskComms {
    pub(crate) fn new(p0: Sender<AnyEvent>, p1: Option<Recipient<FilesChanged>>) -> TaskComms {
        Self {
            any_event_sender: p0,
            servers_recip: p1,
        }
    }
}

#[derive(Debug)]
pub enum Task {
    Runnable(Runnable),
    Group(TaskGroup),
}

impl AsActor for Task {
    fn into_actor2(self: Box<Self>) -> Recipient<TaskCommand> {
        match *self {
            Task::Group(group) => {
                let group_runner = TaskGroupRunner::new(group);
                let s = group_runner.start();
                s.recipient()
            }
            Task::Runnable(Runnable::Many(runner)) => {
                let group = TaskGroup::from(runner);
                let group_runner = TaskGroupRunner::new(group);
                let s = group_runner.start();
                s.recipient()
            }
            Task::Runnable(other_runnable) => Box::new(other_runnable).into_actor2(),
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
    fn from(runner: Runner) -> Self {
        let boxed_tasks = runner
            .tasks
            .into_iter()
            .map(|x| -> Box<dyn AsActor> {
                match x {
                    Runnable::Many(runner) => Box::new(Task::Group(TaskGroup::from(runner))),
                    _ => Box::new(x),
                }
            })
            .collect::<Vec<Box<dyn AsActor>>>();
        match runner.kind {
            RunKind::Sequence => Self::seq(boxed_tasks),
            RunKind::Overlapping => Self::all(boxed_tasks),
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
    pub fn all(tasks: Vec<Box<dyn AsActor>>) -> Self {
        Self {
            run_kind: RunKind::Overlapping,
            tasks,
        }
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
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
    type Result = ResponseActFuture<Self, TaskResult>;

    fn handle(&mut self, msg: TaskCommand, _ctx: &mut Self::Context) -> Self::Result {
        tracing::debug!(done = self.done, "TaskGroupRunner::TaskCommand");
        let Some(group) = self.task_group.take() else {
            todo!("how to handle a concurrent request here?");
        };
        tracing::debug!("  └── {} tasks in group", group.len());
        tracing::debug!("  └── {:?} group run_kind", group.run_kind);

        let future = async move {
            let actors = group
                .tasks
                .into_iter()
                .map(|x| x.into_actor2())
                .collect::<Vec<_>>();
            let mut done = vec![];
            match group.run_kind {
                RunKind::Sequence => {
                    for (index, x) in actors.iter().enumerate() {
                        match x.send(msg.clone()).await {
                            Ok(_) => {
                                tracing::trace!("did send");
                                done.push(index)
                            }
                            Err(e) => tracing::error!("{e}"),
                        }
                    }
                }
                RunKind::Overlapping => {
                    let futs = actors
                        .into_iter()
                        .enumerate()
                        .map(|(index, a)| a.send(msg.clone()).map(move |r| (index, r)));
                    let mut set = JoinSet::from_iter(futs);
                    while let Some(res) = set.join_next().await {
                        match res {
                            Ok((index, _result)) => done.push(index),
                            Err(_) => tracing::error!("could not push index"),
                        }
                    }
                }
            };
            done
        };
        // let self_addr = ctx.address();
        Box::pin(future.into_actor(self).map(|res, actor, _ctx| {
            actor.done = true;
            let child_results = res
                .into_iter()
                .map(|index| TaskResult::ok(index as u64))
                .collect::<Vec<_>>();
            TaskResult::ok_tasks(0, child_results)
        }))
    }
}
