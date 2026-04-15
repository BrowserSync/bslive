use crate::NodePath;
use crate::invocation_result::InvocationResult;
use crate::task_trigger::TaskTrigger;
use std::fmt::Debug;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "InvocationResult")]
pub struct Invocation {
    node_path: NodePath,
    trigger: TaskTrigger,
}

impl Invocation {
    pub fn trigger(&self) -> &TaskTrigger {
        &self.trigger
    }
    pub fn path(&self) -> &NodePath {
        &self.node_path
    }
}

impl Invocation {
    pub fn new(node_path: &NodePath, trigger: TaskTrigger) -> Self {
        Self {
            node_path: node_path.to_owned(),
            trigger,
        }
    }
}
