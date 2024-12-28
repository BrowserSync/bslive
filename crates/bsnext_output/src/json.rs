use crate::OutputWriter;
use bsnext_dto::internal::{InternalEvents, InternalEventsDTO};
use bsnext_dto::ExternalEventsDTO;
use std::io::Write;

pub struct JsonPrint;

impl OutputWriter for JsonPrint {
    fn handle_external_event<W: Write>(
        &self,
        sink: &mut W,
        evt: &ExternalEventsDTO,
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
            InternalEvents::InputError(_) => {}
            InternalEvents::StartupError(_) => {}
        }
        Ok(())
    }
}
