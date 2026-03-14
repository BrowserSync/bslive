use crate::invocation_result::InvocationResult;
use bsnext_fs::FsEventContext;
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "InvocationResult")]
pub struct TaskTrigger {
    trigger_source: TaskTriggerSource,
}

#[derive(Debug, Clone)]
pub struct FsChangesTrigger {
    pub changes: Vec<PathBuf>,
    pub fs_event_context: FsEventContext,
}

#[derive(Debug, Clone)]
pub struct ExecTrigger;

#[derive(Debug, Clone)]
pub enum TaskTriggerSource {
    FsChanges(FsChangesTrigger),
    Exec(ExecTrigger),
}

impl TaskTrigger {
    pub fn new(trigger_source: TaskTriggerSource) -> Self {
        Self { trigger_source }
    }
    pub fn source(&self) -> &TaskTriggerSource {
        &self.trigger_source
    }
}
