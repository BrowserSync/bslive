use crate::OutputWriter;
use std::io::Write;
use bsnext_dto::ExternalEvents;

pub struct JsonPrint;

impl OutputWriter for JsonPrint {
    fn handle_event<W: Write>(&self, sink: &mut W, evt: &ExternalEvents) -> anyhow::Result<()> {
        write!(sink, "{}", serde_json::to_string(&evt)?).map_err(|e| anyhow::anyhow!(e.to_string()))
    }
}
