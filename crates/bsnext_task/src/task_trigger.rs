use crate::task_report::TaskResult;
use bsnext_fs::FsEventContext;
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "TaskResult")]
pub struct TaskTrigger {
    pub variant: TaskTriggerSource,
    pub invocation_id: u64,
}

#[derive(Debug, Clone)]
pub enum TaskTriggerSource {
    FsChanges {
        changes: Vec<PathBuf>,
        fs_event_context: FsEventContext,
    },
    Exec,
}

impl TaskTrigger {
    pub fn new(variant: TaskTriggerSource, invocation_id: u64) -> Self {
        Self {
            variant,
            invocation_id,
        }
    }
    pub fn id(&self) -> u64 {
        self.invocation_id
    }
}
