use crate::as_actor::AsActor;
use crate::invocation::Invocation;
use actix::Recipient;
use std::fmt::{Display, Formatter};

/// `TaskEntry` is a wrapper that enables polymorphic handling of tasks within the system.
///
/// At a high level, it pairs a unique identifier with a boxed task that implements the [`AsActor`] trait.
/// This allows different types of tasks (such as individual operations or entire task groups)
/// to be stored in the same collection (like a `Vec<TaskEntry>`) and treated uniformly when
/// they need to be converted into Actix actors.
///
/// New contributors should think of this as a "standardized container" for anything that can
/// eventually be run as a task.
#[derive(Debug)]
pub struct TaskEntry {
    task: Box<dyn AsActor>,
    id: u64,
}

impl Display for TaskEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "TaskEntry")
    }
}

impl TaskEntry {
    pub fn new(t: Box<dyn AsActor>, id: u64) -> Self {
        Self { id, task: t }
    }
}

impl AsActor for TaskEntry {
    fn into_task_recipient(self: Box<Self>) -> Recipient<Invocation> {
        self.task.into_task_recipient()
    }
}

impl TaskEntry {
    pub fn id(&self) -> u64 {
        self.id
    }
    pub fn sqid(&self) -> String {
        let sqids = sqids::Sqids::default();
        let sqid = sqids.encode(&[self.id]).unwrap();
        sqid.get(0..6).unwrap_or(&sqid).to_string()
    }
}
