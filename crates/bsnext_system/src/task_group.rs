use crate::as_actor::AsActor;
use crate::dyn_item::DynItem;
use crate::task_group_runner::TaskGroupRunner;
use crate::task_list::{OverlappingOpts, RunKind, Runnable, SequenceOpts, TaskList};
use crate::task_trigger::TaskTrigger;
use actix::{Actor, Recipient};

#[derive(Debug)]
pub struct TaskGroup {
    run_kind: RunKind,
    tasks: Vec<DynItem>,
}

impl AsActor for TaskGroup {
    fn into_task_recipient(self: Box<Self>) -> Recipient<TaskTrigger> {
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
            .map(|(i, x)| -> DynItem {
                let item_id = x.as_id_with(i as u64);
                match x {
                    Runnable::Many(runner) => DynItem {
                        task: Box::new(TaskGroup::from(runner)),
                        id: item_id,
                    },
                    _ => DynItem {
                        id: item_id,
                        task: Box::new(x),
                    },
                }
            })
            .collect::<Vec<DynItem>>();
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
    pub fn tasks(self) -> Vec<DynItem> {
        self.tasks
    }
    pub fn seq(tasks: Vec<DynItem>, opts: SequenceOpts, _id: u64) -> Self {
        Self {
            run_kind: RunKind::Sequence { opts },
            tasks,
        }
    }
    pub fn all(tasks: Vec<DynItem>, opts: OverlappingOpts, _id: u64) -> Self {
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
