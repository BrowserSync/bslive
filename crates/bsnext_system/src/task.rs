use crate::as_actor::AsActor;
use crate::task_group::TaskGroup;
use crate::task_group_runner::TaskGroupRunner;
use crate::task_list::Runnable;
use crate::task_trigger::TaskTrigger;
use actix::{Actor, Recipient};
use std::fmt::{Display, Formatter};

#[derive(Debug)]
pub enum Task {
    Runnable(Runnable),
    Group(TaskGroup),
}

impl Display for Task {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Task::Runnable(_) => write!(f, "Task::Runnable"),
            Task::Group(_) => write!(f, "Task::Group"),
        }
    }
}

impl AsActor for Task {
    fn into_task_recipient(self: Box<Self>) -> Recipient<TaskTrigger> {
        match *self {
            Task::Group(group) => {
                let group_runner = TaskGroupRunner::new(group);
                let s = group_runner.start();
                s.recipient()
            }
            Task::Runnable(Runnable::Many(runner)) => {
                let group = TaskGroup::from(runner);
                let group_runner = TaskGroupRunner::new(group);
                let s = group_runner.start();
                s.recipient()
            }
            Task::Runnable(other_runnable) => Box::new(other_runnable).into_task_recipient(),
        }
    }
}
