use crate::runner::{RunKind, Runnable, Runner};
use crate::task::{AsActor, Task, TaskCommand};
use actix::Recipient;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct TaskGroup {
    run_kind: RunKind,
    tasks: Vec<DynItem>,
}

#[derive(Debug)]
pub struct DynItem {
    task: Box<dyn AsActor>,
    id: u64,
}

impl Display for DynItem {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "DynItem")
    }
}

impl DynItem {
    pub fn new(t: Box<dyn AsActor>, id: u64) -> Self {
        Self { id, task: t }
    }
}

impl AsActor for DynItem {
    fn into_actor2(self: Box<Self>) -> Recipient<TaskCommand> {
        self.task.into_actor2()
    }
}

impl DynItem {
    pub fn id(&self) -> u64 {
        self.id
    }
}

// impl Hash for TaskGroup {
//     fn hash<H: Hasher>(&self, state: &mut H) {
//         self.run_kind.hash(state);
//         let ids = self.tasks.iter().map(|x| x.as_id()).collect::<Vec<_>>();
//         ids.hash(state);
//     }
// }

impl From<Runner> for TaskGroup {
    fn from(runner: Runner) -> Self {
        let top_id = runner.as_id();
        let boxed_tasks = runner
            .tasks
            .into_iter()
            .enumerate()
            .map(|(i, x)| -> DynItem {
                let item_id = x.as_id_with(i as u64);
                match x {
                    Runnable::Many(runner) => DynItem {
                        task: Box::new(Task::Group(TaskGroup::from(runner))),
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
            RunKind::Sequence => Self::seq(boxed_tasks, top_id),
            RunKind::Overlapping => Self::all(boxed_tasks, top_id),
        }
    }
}

// impl From<Runner> for TaskGroup {
//     fn from(value: &Runner) -> Self {
//         TaskGroup::from(value.clone())
//     }
// }

impl TaskGroup {
    pub fn run_kind(&self) -> &RunKind {
        &self.run_kind
    }
    pub fn tasks(self) -> Vec<DynItem> {
        self.tasks
    }
    pub fn seq(tasks: Vec<DynItem>, _id: u64) -> Self {
        Self {
            run_kind: RunKind::Sequence,
            tasks,
        }
    }
    pub fn all(tasks: Vec<DynItem>, _id: u64) -> Self {
        Self {
            run_kind: RunKind::Overlapping,
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
