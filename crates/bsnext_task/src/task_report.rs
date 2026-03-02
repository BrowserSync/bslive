use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub enum TaskConclusion {
    Ok(TaskOk),
    Cancelled,
    Err(TaskError),
}

#[derive(Debug, Clone)]
pub struct TaskOk;
#[derive(Debug, Clone)]
pub struct ActualLen(pub usize);
#[derive(Debug, Clone)]
pub struct ExpectedLen(pub usize);

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
pub struct InvocationId(pub u64);

#[derive(Debug, Clone)]
pub struct ExitCode(pub i32);

#[derive(Debug, Clone)]
pub struct TaskReport {
    pub result: TaskResult,
    pub id: u64,
}

impl Display for TaskReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "id: {}", self.id)
    }
}

#[derive(Debug, Clone)]
pub struct TaskResult {
    #[allow(dead_code)]
    pub conclusion: TaskConclusion,
    #[allow(dead_code)]
    pub invocation_id: InvocationId,
    #[allow(dead_code)]
    pub task_reports: Vec<TaskReport>,
}

impl TaskResult {
    pub fn cancelled() -> Self {
        Self {
            task_reports: vec![],
            conclusion: TaskConclusion::Cancelled,
            invocation_id: InvocationId(0),
        }
    }
}

impl Display for TaskResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.conclusion {
            TaskConclusion::Ok(_s) => write!(f, "✅"),
            TaskConclusion::Err(err) => write!(f, "❌, {err}"),
            TaskConclusion::Cancelled => write!(f, "[cancelled]"),
        }
    }
}

impl TaskReport {
    pub fn has_errors(&self) -> bool {
        !self.result.is_ok()
    }
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
        self.result.reports()
    }
    pub fn is_ok(&self) -> bool {
        self.result.is_ok()
    }
}

impl TaskResult {
    pub fn ok(id: InvocationId) -> Self {
        Self {
            conclusion: TaskConclusion::Ok(TaskOk),
            invocation_id: id,
            task_reports: vec![],
        }
    }
    pub fn is_ok(&self) -> bool {
        matches!(self.conclusion, TaskConclusion::Ok(..))
    }
    pub fn err_code(id: InvocationId, code: ExitCode) -> Self {
        Self {
            conclusion: TaskConclusion::Err(TaskError::FailedCode { code }),
            invocation_id: id,
            task_reports: vec![],
        }
    }
    pub fn err_message(id: InvocationId, message: &str) -> Self {
        Self {
            conclusion: TaskConclusion::Err(TaskError::FailedMsg(message.to_string())),
            invocation_id: id,
            task_reports: vec![],
        }
    }
    pub fn timeout(id: InvocationId) -> Self {
        Self {
            conclusion: TaskConclusion::Err(TaskError::FailedTimeout),
            invocation_id: id,
            task_reports: vec![],
        }
    }
    pub fn ok_tasks(id: InvocationId, tasks: Vec<TaskReport>) -> Self {
        Self {
            conclusion: TaskConclusion::Ok(TaskOk),
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
            conclusion: TaskConclusion::Err(TaskError::GroupFailed {
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
            conclusion: TaskConclusion::Err(TaskError::GroupPartial {
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
