use crate::task_report::TaskResult;
use crate::task_trigger::TaskTrigger;
use std::fmt::Debug;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "TaskResult")]
pub struct Invocation {
    pub id: u64,
    pub trigger: TaskTrigger,
}

impl Invocation {
    pub fn new(id: u64, trigger: TaskTrigger) -> Self {
        Self { id, trigger }
    }
    pub fn sqid(&self) -> String {
        let sqids = sqids::Sqids::default();
        sqids
            .encode(&[self.id])
            .unwrap_or_else(|_| self.id.to_string())
    }
}
