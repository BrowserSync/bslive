use crate::invocation_result::InvocationResult;
use crate::sqid;
use crate::task_trigger::TaskTrigger;
use std::fmt::{Debug, Display, Formatter};

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "InvocationResult")]
pub struct Invocation {
    pub id: InvocationId,
    pub trigger: TaskTrigger,
}

impl Invocation {
    pub fn new(id: InvocationId, trigger: TaskTrigger) -> Self {
        Self { id, trigger }
    }
    pub fn sqid(&self) -> String {
        self.id.sqid()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct InvocationId(u64);

impl InvocationId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    pub fn u64(&self) -> u64 {
        self.0
    }
    pub fn sqid(&self) -> String {
        sqid(self.0)
    }
}

impl Display for InvocationId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvocationId({})", self.0)
    }
}
