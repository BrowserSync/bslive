use crate::invocation_result::InvocationResult;
use bsnext_fs::FsEventContext;
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "InvocationResult")]
pub struct TaskTrigger {
    pub trigger_source: TaskTriggerSource,
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
    pub fn new(trigger_source: TaskTriggerSource) -> Self {
        Self { trigger_source }
    }
}
