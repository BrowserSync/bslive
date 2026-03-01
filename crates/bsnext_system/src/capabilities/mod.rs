pub mod output_channel;

use actix::Actor;
use bsnext_dto::internal::AnyEvent;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Capabilities {
    any_event_sender: Sender<AnyEvent>,
}

impl Capabilities {
    pub fn new(sender: Sender<AnyEvent>) -> Self {
        Self {
            any_event_sender: sender,
        }
    }
}

impl Actor for Capabilities {
    type Context = actix::Context<Self>;
}

pub struct TaggedEvent {
    event: AnyEvent,
    id: u64,
}

impl TaggedEvent {
    pub fn sqid(&self) -> String {
        let sqids = sqids::Sqids::default();
        let sqid = sqids.encode(&[self.id]).unwrap();
        sqid.get(0..6).unwrap_or(&sqid).to_string()
    }
}

impl TaggedEvent {
    pub fn new(id: u64, event: AnyEvent) -> TaggedEvent {
        Self { event, id }
    }
}
