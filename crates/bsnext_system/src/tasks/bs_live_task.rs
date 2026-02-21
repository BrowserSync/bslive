use std::fmt::{Display, Formatter};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum BsLiveTask {
    NotifyServer,
    PublishExternalEvent,
}

impl Display for BsLiveTask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BsLiveTask::NotifyServer => write!(f, "BsLiveTask::NotifyServer"),
            BsLiveTask::PublishExternalEvent => write!(f, "BsLiveTask::PublishExternalEvent"),
        }
    }
}
