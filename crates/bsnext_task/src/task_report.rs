use crate::invocation::SpecId;
use crate::invocation_result::InvocationResult;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct TaskReport {
    pub result: InvocationResult,
    pub spec_id: SpecId,
}

impl Display for TaskReport {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "spec_id: {}", self.spec_id)
    }
}

impl TaskReport {
    pub fn has_errors(&self) -> bool {
        !self.result.is_ok()
    }
}

impl TaskReport {
    pub fn new(result: InvocationResult, spec_id: SpecId) -> Self {
        Self { spec_id, result }
    }
    pub fn spec_id(&self) -> u64 {
        self.spec_id.u64()
    }
    pub fn result(&self) -> &InvocationResult {
        &self.result
    }
    pub fn reports(&self) -> &[TaskReport] {
        self.result.reports()
    }
    pub fn is_ok(&self) -> bool {
        self.result.is_ok()
    }
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
pub struct ExitCode(pub i32);
