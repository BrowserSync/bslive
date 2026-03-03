use crate::invocation::InvocationId;
use crate::task_report::{ActualLen, ExitCode, ExpectedLen, TaskError, TaskOk, TaskReport};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct InvocationResult {
    #[allow(dead_code)]
    pub conclusion: InvocationConclusion,
    #[allow(dead_code)]
    pub invocation_id: InvocationId,
    #[allow(dead_code)]
    pub task_reports: Vec<TaskReport>,
}

impl InvocationResult {
    pub fn cancelled() -> Self {
        Self {
            task_reports: vec![],
            conclusion: InvocationConclusion::Cancelled,
            invocation_id: InvocationId(0),
        }
    }
}

impl Display for InvocationResult {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match &self.conclusion {
            InvocationConclusion::Ok(_s) => write!(f, "✅"),
            InvocationConclusion::Err(err) => write!(f, "❌, {err}"),
            InvocationConclusion::Cancelled => write!(f, "[cancelled]"),
        }
    }
}

impl InvocationResult {
    pub fn ok(id: InvocationId) -> Self {
        Self {
            conclusion: InvocationConclusion::Ok(TaskOk),
            invocation_id: id,
            task_reports: vec![],
        }
    }
    pub fn is_ok(&self) -> bool {
        matches!(self.conclusion, InvocationConclusion::Ok(..))
    }
    pub fn err_code(id: InvocationId, code: ExitCode) -> Self {
        Self {
            conclusion: InvocationConclusion::Err(TaskError::FailedCode { code }),
            invocation_id: id,
            task_reports: vec![],
        }
    }
    pub fn err_message(id: InvocationId, message: &str) -> Self {
        Self {
            conclusion: InvocationConclusion::Err(TaskError::FailedMsg(message.to_string())),
            invocation_id: id,
            task_reports: vec![],
        }
    }
    pub fn timeout(id: InvocationId) -> Self {
        Self {
            conclusion: InvocationConclusion::Err(TaskError::FailedTimeout),
            invocation_id: id,
            task_reports: vec![],
        }
    }
    pub fn ok_tasks(id: InvocationId, tasks: Vec<TaskReport>) -> Self {
        Self {
            conclusion: InvocationConclusion::Ok(TaskOk),
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
            conclusion: InvocationConclusion::Err(TaskError::GroupFailed {
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
            conclusion: InvocationConclusion::Err(TaskError::GroupPartial {
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
    pub fn to_report_and_map(self, id: u64) -> (TaskReport, HashMap<u64, TaskReport>) {
        let report = self.to_report(id);
        let mut report_map = HashMap::new();
        every_report(&mut report_map, &report);
        (report, report_map)
    }
    pub fn reports(&self) -> &[TaskReport] {
        &self.task_reports
    }
}

pub fn every_report(hm: &mut HashMap<u64, TaskReport>, report: &TaskReport) {
    hm.insert(report.id(), report.clone());
    for inner in &report.result().task_reports {
        every_report(hm, inner)
    }
}

#[derive(Debug, Clone)]
pub enum InvocationConclusion {
    Ok(TaskOk),
    Cancelled,
    Err(TaskError),
}
