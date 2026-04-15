use crate::invocation_result::InvocationResult;
use bsnext_fs::FsEventContext;
use std::fmt::Debug;
use std::path::PathBuf;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "InvocationResult")]
pub struct TaskTrigger {
    trigger_source: TaskTriggerSource,
}

impl TaskTrigger {
    pub fn new(trigger_source: TaskTriggerSource) -> Self {
        Self { trigger_source }
    }
    pub fn source(&self) -> &TaskTriggerSource {
        &self.trigger_source
    }
}

#[derive(Debug, Clone)]
pub enum TaskTriggerSource {
    FsChanges(FsChangesTrigger),
    Exec(ExecTrigger),
}

#[derive(Debug, Clone)]
pub struct FsChangesTrigger {
    changes: Vec<PathBuf>,
    fs_event_context: FsEventContext,
}

impl FsChangesTrigger {
    pub fn new(changes: Vec<PathBuf>, fs_event_context: FsEventContext) -> Self {
        Self {
            fs_event_context,
            changes,
        }
    }
    pub fn fs_ctx(&self) -> &FsEventContext {
        &self.fs_event_context
    }
    pub fn changes(&self) -> &Vec<PathBuf> {
        &self.changes
    }
}

#[derive(Debug, Clone)]
pub struct ExecTrigger;
