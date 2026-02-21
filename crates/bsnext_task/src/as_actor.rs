use crate::invocation::Invocation;
use actix::Recipient;

pub trait AsActor: std::fmt::Debug {
    fn into_task_recipient(self: Box<Self>) -> Recipient<Invocation>;
}
