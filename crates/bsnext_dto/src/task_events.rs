use crate::any_event::AnyEvent;
use crate::archy::ArchyNode;
use crate::external_events::{
    ExternalEventsDTO, InvocationIdDTO, TaskActionDTO, TaskActionStageDTO, TaskConclusionDTO,
    TaskReportDTO, TaskResultDTO,
};
use bsnext_task::invocation_result::{InvocationConclusion, InvocationResult};
use bsnext_task::task_report::TaskReport;
use bsnext_task::NodePath;
use std::collections::HashMap;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct TaskReportAndTree {
    pub report: TaskReport,
    pub tree: ArchyNode,
    pub report_map: HashMap<NodePath, TaskReport>,
}

#[derive(Debug, Clone)]
pub struct TaskAction {
    pub id: u64,
    pub stage: TaskActionStage,
}

#[derive(Debug, Clone)]
pub enum TaskActionStage {
    Started { tree: ArchyNode },
    Ended { tree: ArchyNode, report: TaskReport },
    Error,
}

impl TaskActionStage {
    pub fn started(tree: ArchyNode) -> AnyEvent {
        // let action = TaskAction {
        //     id,
        //     stage: TaskActionStage::Started { tree },
        // };
        let dto = TaskActionDTO {
            stage: TaskActionStageDTO::Started { tree },
        };
        AnyEvent::External(ExternalEventsDTO::TaskAction(dto))
    }
    pub fn complete(
        tree: ArchyNode,
        report: TaskReport,
        report_map: HashMap<NodePath, TaskReport>,
    ) -> AnyEvent {
        let report_map_dto = report_map
            .iter()
            .map(|(k, v)| (k.to_string(), TaskReportDTO::from(v.clone())))
            .collect();
        let dto = TaskActionDTO {
            stage: TaskActionStageDTO::Ended {
                report_map: report_map_dto,
                tree,
                report: TaskReportDTO::from(report),
            },
        };
        AnyEvent::External(ExternalEventsDTO::TaskAction(dto))
    }
}

impl From<TaskReport> for TaskReportDTO {
    fn from(value: TaskReport) -> Self {
        TaskReportDTO {
            result: TaskResultDTO::from(value.result),
        }
    }
}

impl From<InvocationResult> for TaskResultDTO {
    fn from(value: InvocationResult) -> Self {
        TaskResultDTO {
            invocation_id: InvocationIdDTO(value.node_path.to_string()),
            conclusion: match value.conclusion {
                InvocationConclusion::Ok(_) => TaskConclusionDTO::Ok,
                InvocationConclusion::Err(e) => TaskConclusionDTO::Err(e.to_string()),
                InvocationConclusion::Cancelled => TaskConclusionDTO::Cancelled,
            },
            task_reports: value
                .task_reports
                .into_iter()
                .map(TaskReportDTO::from)
                .collect(),
        }
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub enum InitialTaskError {
    #[error("initial tasks did not complete, as determined from report. TODO: access report here for better errors")]
    FailedReport,
    #[error("Task(s) {expected} not found. {available}")]
    MissingTask {
        expected: Expected,
        available: Available,
    },
    #[error("initial tasks did not complete, for an unknown reason")]
    FailedUnknown,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub struct Expected(pub Vec<String>);

impl Display for Expected {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            self.0
                .iter()
                .map(|x| format!("\"{}\"", x))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, thiserror::Error)]
pub struct Available(pub Vec<String>);

impl Display for Available {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.0.is_empty() {
            return Ok(());
        }
        write!(
            f,
            "Available: {}",
            self.0
                .iter()
                .map(|x| format!("\"{}\"", x))
                .collect::<Vec<_>>()
                .join(", ")
        )
    }
}
