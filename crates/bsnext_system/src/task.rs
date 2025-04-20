use crate::runner::{Runnable, TreeDisplay};
use crate::task_group::TaskGroup;
use crate::task_group_runner::TaskGroupRunner;
use actix::{Actor, Handler, Recipient};
use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;
use bsnext_dto::internal::AnyEvent;
use bsnext_dto::OutputLineDTO;
use bsnext_fs::FsEventContext;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};
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
        output: Vec<OutputLineDTO>,
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
    pub(crate) status: TaskStatus,
    #[allow(dead_code)]
    invocation_id: InvocationId,
    #[allow(dead_code)]
    pub(crate) task_reports: Vec<TaskReport>,
}

impl Display for TaskResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.status {
            TaskStatus::Ok(s) => write!(f, "✅"),
            TaskStatus::Err(err) => write!(f, "❌, {}", err),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskReport {
    result: TaskResult,
    id: u64,
}

impl TaskReport {
    pub fn new(result: TaskResult, id: u64) -> Self {
        Self { id, result }
    }
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn result(&self) -> &TaskResult {
        &self.result
    }
    pub fn reports(&self) -> &[TaskReport] {
        &self.result.reports()
    }
    pub fn is_ok(&self) -> bool {
        self.result.is_ok()
    }
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
            task_reports: vec![],
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
            task_reports: vec![],
        }
    }
    pub fn err_message(id: InvocationId, message: &str) -> Self {
        Self {
            status: TaskStatus::Err(TaskError::FailedMsg(message.to_string())),
            invocation_id: id,
            task_reports: vec![],
        }
    }
    pub fn timeout(id: InvocationId) -> Self {
        Self {
            status: TaskStatus::Err(TaskError::FailedTimeout),
            invocation_id: id,
            task_reports: vec![],
        }
    }
    pub fn ok_tasks(id: InvocationId, tasks: Vec<TaskReport>) -> Self {
        Self {
            status: TaskStatus::Ok(TaskOk),
            invocation_id: id,
            task_reports: tasks,
        }
    }
    pub fn err_tasks(
        id: InvocationId,
        failed_only: Vec<TaskReport>,
        results: Vec<TaskReport>,
    ) -> Self {
        Self {
            status: TaskStatus::Err(TaskError::GroupFailed {
                failed_tasks: failed_only.clone(),
            }),
            invocation_id: id,
            task_reports: results,
        }
    }
    pub fn err_partial_tasks(
        id: InvocationId,
        tasks: Vec<TaskReport>,
        expected: ExpectedLen,
    ) -> Self {
        Self {
            status: TaskStatus::Err(TaskError::GroupPartial {
                actual: ActualLen(tasks.len()),
                expected,
                failed_tasks: tasks.clone(),
            }),
            invocation_id: id,
            task_reports: tasks,
        }
    }
    pub fn to_report(self, id: u64) -> TaskReport {
        TaskReport { id, result: self }
    }
    pub fn reports(&self) -> &[TaskReport] {
        &self.task_reports
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
    #[error("group failed")]
    GroupFailed { failed_tasks: Vec<TaskReport> },
    #[error("expected {} task results, only seen {}", expected.0, actual.0)]
    GroupPartial {
        expected: ExpectedLen,
        actual: ActualLen,
        failed_tasks: Vec<TaskReport>,
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

impl Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Task::Runnable(_) => write!(f, "Task::Runnable"),
            Task::Group(_) => write!(f, "Task::Group"),
        }
    }
}

impl TreeDisplay for Task {}
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
    // fn as_id(self: Box<Self>) -> u64 {
    //     let mut hasher = DefaultHasher::new();
    //     match *self {
    //         Task::Runnable(r) => r.hash(&mut hasher),
    //         Task::Group(g) => g.hash(&mut hasher),
    //     }
    //     hasher.finish()
    // }
}

pub trait AsActor: std::fmt::Debug + TreeDisplay {
    fn into_actor2(self: Box<Self>) -> Recipient<TaskCommand>;
}
