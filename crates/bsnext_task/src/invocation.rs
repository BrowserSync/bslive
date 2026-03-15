use crate::invocation_result::InvocationResult;
use crate::task_trigger::TaskTrigger;
use std::fmt::{Debug, Display, Formatter};

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "InvocationResult")]
pub struct Invocation {
    spec_id: SpecId,
    trigger: TaskTrigger,
}

impl Invocation {
    pub fn trigger(&self) -> &TaskTrigger {
        &self.trigger
    }
    pub fn spec_id(&self) -> &SpecId {
        &self.spec_id
    }
}

impl Invocation {
    pub fn new(spec_id: SpecId, trigger: TaskTrigger) -> Self {
        Self { spec_id, trigger }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SpecId(u64);

impl SpecId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
    pub fn u64(&self) -> u64 {
        self.0
    }
}

impl Display for SpecId {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "InvocationId({})", self.0)
    }
}
