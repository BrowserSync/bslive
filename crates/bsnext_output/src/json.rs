use crate::OutputWriter;
use bsnext_dto::internal::{InternalEvents, InternalEventsDTO};
use bsnext_dto::{ExternalEvents, StartupEvent};
use std::io::Write;

pub struct JsonPrint;

impl OutputWriter for JsonPrint {
    fn handle_external_event<W: Write>(
        &self,
        sink: &mut W,
        evt: &ExternalEvents,
    ) -> anyhow::Result<()> {
        writeln!(sink, "{}", serde_json::to_string(&evt)?)
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    }

    fn handle_internal_event<W: Write>(
        &self,
        sink: &mut W,
        evt: InternalEvents,
    ) -> anyhow::Result<()> {
        match evt {
            InternalEvents::ServersChanged { server_resp, .. } => {
                let output = InternalEventsDTO::ServersChanged(server_resp);
                writeln!(sink, "{}", serde_json::to_string(&output)?)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            }
        }
        Ok(())
    }

    fn handle_startup_event<W: Write>(
        &self,
        sink: &mut W,
        evt: &StartupEvent,
    ) -> anyhow::Result<()> {
        writeln!(sink, "{}", serde_json::to_string(&evt)?)
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    }
}
