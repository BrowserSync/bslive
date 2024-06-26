use crate::json::JsonPrint;
use crate::pretty::PrettyPrint;
use bsnext_dto::{ExternalEvents, StartupEvent};
use std::io::Write;

pub mod json;
pub mod pretty;
#[cfg(test)]
mod tests;

pub trait OutputWriter {
    fn handle_external_event<W: Write>(
        &self,
        sink: &mut W,
        evt: &ExternalEvents,
    ) -> anyhow::Result<()>;
    fn handle_startup_event<W: Write>(
        &self,
        sink: &mut W,
        evt: &StartupEvent,
    ) -> anyhow::Result<()>;
}

pub enum Writers {
    Pretty,
    Json,
}

impl OutputWriter for Writers {
    fn handle_external_event<W: Write>(
        &self,
        sink: &mut W,
        evt: &ExternalEvents,
    ) -> anyhow::Result<()> {
        match self {
            Writers::Pretty => PrettyPrint.handle_external_event(sink, evt),
            Writers::Json => JsonPrint.handle_external_event(sink, evt),
        }
    }

    fn handle_startup_event<W: Write>(
        &self,
        sink: &mut W,
        evt: &StartupEvent,
    ) -> anyhow::Result<()> {
        match self {
            Writers::Pretty => PrettyPrint.handle_startup_event(sink, evt),
            Writers::Json => JsonPrint.handle_startup_event(sink, evt),
        }
    }
}
