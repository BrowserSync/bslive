use actix::Recipient;
use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;
use bsnext_dto::internal::{AnyEvent, TaskResult};
use bsnext_fs::FsEventContext;
use std::fmt::Debug;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "TaskResult")]
pub enum TaskTrigger {
    FsChanges {
        changes: Vec<PathBuf>,
        fs_event_context: FsEventContext,
        task_comms: TaskComms,
        invocation_id: u64,
    },
    Exec {
        task_comms: TaskComms,
        invocation_id: u64,
    },
}

impl TaskTrigger {
    pub fn comms(&self) -> &TaskComms {
        match self {
            TaskTrigger::FsChanges { task_comms, .. } => task_comms,
            TaskTrigger::Exec { task_comms, .. } => task_comms,
        }
    }
    pub fn id(&self) -> u64 {
        match self {
            TaskTrigger::FsChanges { invocation_id, .. } => *invocation_id,
            TaskTrigger::Exec { invocation_id, .. } => *invocation_id,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskComms {
    pub any_event_sender: Sender<AnyEvent>,
    pub servers_recip: Option<Recipient<FilesChanged>>,
}

impl TaskComms {
    pub(crate) fn new(p0: Sender<AnyEvent>, p1: Option<Recipient<FilesChanged>>) -> TaskComms {
        Self {
            any_event_sender: p0,
            servers_recip: p1,
        }
    }
}
