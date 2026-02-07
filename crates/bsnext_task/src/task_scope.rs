use crate::as_actor::AsActor;
use crate::invocation::Invocation;
use crate::task_entry::TaskEntry;
use crate::task_scope_runner::TaskScopeRunner;
use crate::{OverlappingOpts, RunKind, SequenceOpts};
use actix::{Actor, Recipient};

#[derive(Debug)]
pub struct TaskScope {
    run_kind: RunKind,
    tasks: Vec<TaskEntry>,
}

impl AsActor for TaskScope {
    fn into_task_recipient(self: Box<Self>) -> Recipient<Invocation> {
        let group_runner = TaskScopeRunner::new(*self);
        let s = group_runner.start();
        s.recipient()
    }
}

impl TaskScope {
    pub fn run_kind(&self) -> &RunKind {
        &self.run_kind
    }
    pub fn exit_on_failure(&self) -> bool {
        match self.run_kind() {
            RunKind::Sequence {
                opts: SequenceOpts { exit_on_failure },
            } => *exit_on_failure,
            RunKind::Overlapping {
                opts: OverlappingOpts {
                    exit_on_failure, ..
                },
            } => *exit_on_failure,
        }
    }
    pub fn tasks(self) -> Vec<TaskEntry> {
        self.tasks
    }
    pub fn seq(tasks: Vec<TaskEntry>, opts: SequenceOpts, _id: u64) -> Self {
        Self {
            run_kind: RunKind::Sequence { opts },
            tasks,
        }
    }
    pub fn all(tasks: Vec<TaskEntry>, opts: OverlappingOpts, _id: u64) -> Self {
        Self {
            run_kind: RunKind::Overlapping { opts },
            tasks,
        }
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}
