use crate::runner::{RunKind, Runnable, Runner};
use crate::task_group_runner::TaskGroupRunner;
use actix::{Actor, Handler, Recipient};
use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;
use bsnext_dto::internal::AnyEvent;
use bsnext_dto::OutputLineDTO;
use bsnext_fs::FsEventContext;
use std::fmt::Debug;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "TaskResult")]
pub enum TaskCommand {
    Changes {
        changes: Vec<PathBuf>,
        fs_event_context: FsEventContext,
        task_comms: TaskComms,
        invocation_id: u64,
    },
    Log {
        fs_event_context: FsEventContext,
        task_comms: TaskComms,
        invocation_id: u64,
        output: OutputLineDTO,
    },
}

impl TaskCommand {
    pub fn comms(&self) -> &TaskComms {
        match self {
            TaskCommand::Changes { task_comms, .. } => task_comms,
            TaskCommand::Log { task_comms, .. } => task_comms,
        }
    }
    pub fn id(&self) -> u64 {
        match self {
            TaskCommand::Changes { invocation_id, .. } => *invocation_id,
            TaskCommand::Log { invocation_id, .. } => *invocation_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskResult {
    #[allow(dead_code)]
    status: TaskStatus,
    #[allow(dead_code)]
    invocation_id: InvocationId,
    #[allow(dead_code)]
    task_results: Vec<TaskResult>,
}

#[derive(Debug, Clone)]
pub struct InvocationId(pub(crate) u64);

#[derive(Debug, Clone)]
pub struct ExitCode(pub(crate) i32);

impl TaskResult {
    pub fn ok(id: InvocationId) -> Self {
        Self {
            status: TaskStatus::Ok(TaskOk),
            invocation_id: id,
            task_results: vec![],
        }
    }
    pub fn err(&self) -> Option<&TaskError> {
        match &self.status {
            TaskStatus::Ok(_) => None,
            TaskStatus::Err(e) => Some(&e),
        }
    }
    pub fn is_ok(&self) -> bool {
        matches!(self.status, TaskStatus::Ok(..))
    }
    pub fn err_code(id: InvocationId, code: ExitCode) -> Self {
        Self {
            status: TaskStatus::Err(TaskError::FailedCode { code }),
            invocation_id: id,
            task_results: vec![],
        }
    }
    pub fn err_message(id: InvocationId, message: &str) -> Self {
        Self {
            status: TaskStatus::Err(TaskError::FailedMsg(message.to_string())),
            invocation_id: id,
            task_results: vec![],
        }
    }
    pub fn timeout(id: InvocationId) -> Self {
        Self {
            status: TaskStatus::Err(TaskError::FailedTimeout),
            invocation_id: id,
            task_results: vec![],
        }
    }
    pub fn ok_tasks(id: InvocationId, tasks: Vec<TaskResult>) -> Self {
        Self {
            status: TaskStatus::Ok(TaskOk),
            invocation_id: id,
            task_results: tasks,
        }
    }
    pub fn err_tasks(id: InvocationId, tasks: Vec<TaskResult>) -> Self {
        Self {
            status: TaskStatus::Err(TaskError::GroupFailed {
                task_results: tasks.clone(),
            }),
            invocation_id: id,
            task_results: tasks,
        }
    }
    pub fn err_partial_tasks(
        id: InvocationId,
        tasks: Vec<TaskResult>,
        expected: ExpectedLen,
    ) -> Self {
        Self {
            status: TaskStatus::Err(TaskError::GroupPartial {
                actual: ActualLen(tasks.len()),
                expected,
            }),
            invocation_id: id,
            task_results: tasks,
        }
    }
}

#[derive(Debug, Clone)]
pub enum TaskStatus {
    Ok(TaskOk),
    Err(TaskError),
}

#[derive(Debug, Clone)]
pub struct TaskOk;
#[derive(Debug, Clone)]
pub struct ActualLen(pub(crate) usize);
#[derive(Debug, Clone)]
pub struct ExpectedLen(pub(crate) usize);

#[derive(Debug, Clone, thiserror::Error)]
pub enum TaskError {
    #[error("{0}")]
    FailedMsg(String),
    #[error("failed with code: {0}", code.0)]
    FailedCode { code: ExitCode },
    #[error("timed out")]
    FailedTimeout,
    #[error("{0} failed in group", task_results.len())]
    GroupFailed { task_results: Vec<TaskResult> },
    #[error("expected {} task results, only seen {}", expected.0, actual.0)]
    GroupPartial {
        expected: ExpectedLen,
        actual: ActualLen,
    },
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
    pub run_kind: RunKind,
    pub tasks: Vec<Box<dyn AsActor>>,
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
