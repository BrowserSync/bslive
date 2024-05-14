use crate::OutputWriter;
use bsnext_core::dto::ExternalEvents;
use std::io::Write;

pub struct JsonPrint;

impl OutputWriter for JsonPrint {
    fn handle_event<W: Write>(&self, sink: &mut W, evt: &ExternalEvents) -> anyhow::Result<()> {
        write!(sink, "{}", serde_json::to_string(&evt)?).map_err(|e| anyhow::anyhow!(e.to_string()))
    }
}
