use crate::external_events::ExternalEventsDTO;

#[derive(Debug)]
pub enum AnyEvent {
    External(ExternalEventsDTO),
}
