pub mod output_channel;
pub mod servers_addr;

use actix::{Actor, Addr};
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_dto::internal::AnyEvent;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub struct Capabilities {
    any_event_sender: Sender<AnyEvent>,
    servers_addr: Addr<ServersSupervisor>,
}

impl Capabilities {
    pub fn new(sender: Sender<AnyEvent>, servers: Addr<ServersSupervisor>) -> Self {
        Self {
            any_event_sender: sender,
            servers_addr: servers,
        }
    }
}

impl Actor for Capabilities {
    type Context = actix::Context<Self>;
}

pub struct TaggedEvent {
    event: AnyEvent,
}

impl TaggedEvent {
    pub fn new(event: AnyEvent) -> TaggedEvent {
        Self { event }
    }
}
