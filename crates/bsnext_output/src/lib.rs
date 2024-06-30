use crate::json::JsonPrint;
use crate::pretty::PrettyPrint;
use crate::ratatui::RatatuiSender;
use bsnext_dto::internal::InternalEvents;
use bsnext_dto::{ExternalEvents, StartupEvent};
use std::io::Write;

pub mod json;
pub mod pretty;
pub mod ratatui;
#[cfg(test)]
mod tests;

pub trait OutputWriter {
    fn handle_external_event<W: Write>(
        &self,
        _sink: &mut W,
        _evt: &ExternalEvents,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    fn handle_internal_event<W: Write>(
        &self,
        _sink: &mut W,
        _evt: InternalEvents,
    ) -> anyhow::Result<()> {
        Ok(())
    }
    fn handle_startup_event<W: Write>(
        &self,
        _sink: &mut W,
        _evt: &StartupEvent,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

pub enum Writers {
    Pretty,
    Json,
    Ratatui(RatatuiSender),
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
            Writers::Ratatui(r) => r.handle_external_event(sink, evt),
        }
    }
    fn handle_internal_event<W: Write>(
        &self,
        sink: &mut W,
        evt: InternalEvents,
    ) -> anyhow::Result<()> {
        match self {
            Writers::Pretty => PrettyPrint.handle_internal_event(sink, evt),
            Writers::Json => JsonPrint.handle_internal_event(sink, evt),
            Writers::Ratatui(r) => r.handle_internal_event(sink, evt),
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
            Writers::Ratatui(r) => r.handle_startup_event(sink, evt),
        }
    }
}
