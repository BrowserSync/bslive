use crate::task_trigger::TaskTrigger;
use actix::Recipient;

pub trait AsActor: std::fmt::Debug {
    fn into_task_recipient(self: Box<Self>) -> Recipient<TaskTrigger>;
}
