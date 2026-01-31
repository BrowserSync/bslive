use bsnext_dto::internal::{AnyEvent, TaskResult};
use bsnext_fs::FsEventContext;
use std::fmt::Debug;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "TaskResult")]
pub struct TaskTrigger {
    pub variant: TaskTriggerSource,
    pub comms: TaskComms,
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
    pub fn comms(&self) -> &TaskComms {
        &self.comms
    }
    pub fn id(&self) -> u64 {
        self.invocation_id
    }
}

#[derive(Debug, Clone)]
pub struct TaskComms {
    pub any_event_sender: Sender<AnyEvent>,
}

impl TaskComms {
    pub fn new(p0: Sender<AnyEvent>) -> TaskComms {
        Self {
            any_event_sender: p0,
        }
    }
}
