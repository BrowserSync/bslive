use crate::OutputWriter;
use bsnext_dto::{ExternalEvents, StartupEvent};
use std::io::Write;

pub struct JsonPrint;

impl OutputWriter for JsonPrint {
    fn handle_external_event<W: Write>(
        &self,
        sink: &mut W,
        evt: ExternalEvents,
    ) -> anyhow::Result<()> {
        write!(sink, "{}", serde_json::to_string(&evt)?).map_err(|e| anyhow::anyhow!(e.to_string()))
    }

    fn handle_startup_event<W: Write>(
        &self,
        sink: &mut W,
        evt: &StartupEvent,
    ) -> anyhow::Result<()> {
        write!(sink, "{}", serde_json::to_string(&evt)?).map_err(|e| anyhow::anyhow!(e.to_string()))
    }
}
