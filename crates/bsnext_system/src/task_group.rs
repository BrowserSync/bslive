use crate::as_actor::AsActor;
use crate::task_entry::TaskEntry;
use crate::task_group_runner::TaskGroupRunner;
use crate::task_list::{OverlappingOpts, RunKind, Runnable, SequenceOpts, TaskList};
use crate::tasks::sh_cmd::OneTask;
use actix::{Actor, Recipient};

#[derive(Debug)]
pub struct TaskGroup {
    run_kind: RunKind,
    tasks: Vec<TaskEntry>,
}

impl AsActor for TaskGroup {
    fn into_task_recipient(self: Box<Self>) -> Recipient<OneTask> {
        let group_runner = TaskGroupRunner::new(*self);
        let s = group_runner.start();
        s.recipient()
    }
}

impl From<TaskList> for TaskGroup {
    fn from(runner: TaskList) -> Self {
        let top_id = runner.as_id();
        let boxed_tasks = runner
            .tasks
            .into_iter()
            .enumerate()
            .map(|(i, x)| -> TaskEntry {
                let item_id = x.as_id_with(i as u64);
                match x {
                    Runnable::Many(runner) => TaskEntry {
                        id: item_id,
                        task: Box::new(TaskGroup::from(runner)),
                    },
                    _ => TaskEntry {
                        id: item_id,
                        task: Box::new(x),
                    },
                }
            })
            .collect::<Vec<TaskEntry>>();
        match runner.run_kind {
            RunKind::Sequence { opts } => Self::seq(boxed_tasks, opts, top_id),
            RunKind::Overlapping { opts } => Self::all(boxed_tasks, opts, top_id),
        }
    }
}

impl TaskGroup {
    pub fn run_kind(&self) -> &RunKind {
        &self.run_kind
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
