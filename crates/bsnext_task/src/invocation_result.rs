use crate::task_report::{ActualLen, ExitCode, ExpectedLen, TaskError, TaskOk, TaskReport};
use crate::{ContentId, NodePath};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct InvocationResult {
    #[allow(dead_code)]
    pub conclusion: InvocationConclusion,
    #[allow(dead_code)]
    pub node_path: NodePath,
    #[allow(dead_code)]
    pub task_reports: Vec<TaskReport>,
}

impl InvocationResult {
    pub fn cancelled() -> Self {
        let p = NodePath::root_for(ContentId::new(0));
        Self {
            task_reports: vec![],
            conclusion: InvocationConclusion::Cancelled,
            node_path: p,
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
    pub fn ok(node_path: NodePath) -> Self {
        Self {
            conclusion: InvocationConclusion::Ok(TaskOk),
            node_path,
            task_reports: vec![],
        }
    }
    pub fn is_ok(&self) -> bool {
        matches!(self.conclusion, InvocationConclusion::Ok(..))
    }
    pub fn err_code(node_path: NodePath, code: ExitCode) -> Self {
        Self {
            conclusion: InvocationConclusion::Err(TaskError::FailedCode { code }),
            node_path,
            task_reports: vec![],
        }
    }
    pub fn err_message(node_path: NodePath, message: &str) -> Self {
        Self {
            conclusion: InvocationConclusion::Err(TaskError::FailedMsg(message.to_string())),
            node_path,
            task_reports: vec![],
        }
    }
    pub fn timeout(node_path: NodePath) -> Self {
        Self {
            conclusion: InvocationConclusion::Err(TaskError::FailedTimeout),
            node_path,
            task_reports: vec![],
        }
    }
    pub fn ok_tasks(node_path: NodePath, tasks: Vec<TaskReport>) -> Self {
        Self {
            conclusion: InvocationConclusion::Ok(TaskOk),
            node_path,
            task_reports: tasks,
        }
    }
    pub fn err_tasks(
        node_path: NodePath,
        failed_only: Vec<TaskReport>,
        results: Vec<TaskReport>,
    ) -> Self {
        Self {
            conclusion: InvocationConclusion::Err(TaskError::GroupFailed {
                failed_tasks: failed_only.clone(),
            }),
            node_path,
            task_reports: results,
        }
    }
    pub fn err_partial_tasks(
        node_path: NodePath,
        tasks: Vec<TaskReport>,
        expected: ExpectedLen,
    ) -> Self {
        Self {
            conclusion: InvocationConclusion::Err(TaskError::GroupPartial {
                actual: ActualLen(tasks.len()),
                expected,
                failed_tasks: tasks.clone(),
            }),
            node_path,
            task_reports: tasks,
        }
    }
    pub fn to_report(self, node_path: NodePath) -> TaskReport {
        TaskReport {
            node_path,
            result: self,
        }
    }
    pub fn to_report_and_map(
        self,
        node_path: NodePath,
    ) -> (TaskReport, HashMap<NodePath, TaskReport>) {
        let report = self.to_report(node_path);
        let mut report_map = HashMap::new();
        every_report(&mut report_map, &report);
        (report, report_map)
    }
    pub fn reports(&self) -> &[TaskReport] {
        &self.task_reports
    }
}

pub fn every_report(hm: &mut HashMap<NodePath, TaskReport>, report: &TaskReport) {
    hm.insert(report.node_path(), report.clone());
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
