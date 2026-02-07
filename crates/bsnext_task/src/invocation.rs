use crate::task_trigger::{TaskComms, TaskTrigger};
use bsnext_dto::internal::TaskResult;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "TaskResult")]
pub struct Invocation {
    pub id: u64,
    pub trigger: TaskTrigger,
    pub comms: TaskComms,
}

impl Invocation {
    pub fn new(id: u64, trigger: TaskTrigger, comms: TaskComms) -> Self {
        Self { id, trigger, comms }
    }
    pub fn sqid(&self) -> String {
        let sqids = sqids::Sqids::default();
        sqids
            .encode(&[self.id])
            .unwrap_or_else(|_| self.id.to_string())
    }
}
