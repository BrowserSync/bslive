use actix::Recipient;
use std::path::PathBuf;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
pub enum TaskCommand {
    Changes(Vec<PathBuf>),
}

#[derive(Debug, Clone)]
pub struct TaskManager {
    running: bool,
    addr: Recipient<TaskCommand>,
}

impl TaskManager {
    pub fn new(addr: Recipient<TaskCommand>) -> Self {
        TaskManager {
            addr,
            running: false,
        }
    }

    pub fn addr(&self) -> &Recipient<TaskCommand> {
        &self.addr
    }
}
