use crate::tasks::sh_cmd::OneTask;
use actix::Recipient;

pub trait AsActor: std::fmt::Debug {
    fn into_task_recipient(self: Box<Self>) -> Recipient<OneTask>;
}
