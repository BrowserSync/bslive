use crate::json::JsonPrint;
use crate::pretty::PrettyPrint;
use std::io::Write;
use bsnext_dto::ExternalEvents;

pub mod json;
pub mod pretty;

pub trait OutputWriter {
    fn handle_event<W: Write>(&self, sink: &mut W, evt: &ExternalEvents) -> anyhow::Result<()>;
}

pub enum Writers {
    Pretty,
    Json,
}

impl OutputWriter for Writers {
    fn handle_event<W: Write>(&self, sink: &mut W, evt: &ExternalEvents) -> anyhow::Result<()> {
        match self {
            Writers::Pretty => PrettyPrint.handle_event(sink, evt),
            Writers::Json => JsonPrint.handle_event(sink, evt),
        }
    }
}
