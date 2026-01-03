use crate::as_actor::AsActor;
use crate::task_trigger::TaskTrigger;
use actix::Recipient;
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub struct DynItem {
    pub task: Box<dyn AsActor>,
    pub id: u64,
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
    fn into_task_recipient(self: Box<Self>) -> Recipient<TaskTrigger> {
        self.task.into_task_recipient()
    }
}

impl DynItem {
    pub fn id(&self) -> u64 {
        self.id
    }
}
