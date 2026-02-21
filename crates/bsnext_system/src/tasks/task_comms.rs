use bsnext_dto::internal::AnyEvent;
use tokio::sync::mpsc::Sender;

#[derive(Debug, Clone)]
pub struct TaskComms {
    pub any_event_sender: Sender<AnyEvent>,
}

impl TaskComms {
    pub fn new(p0: Sender<AnyEvent>) -> TaskComms {
        Self {
            any_event_sender: p0,
        }
    }
}
