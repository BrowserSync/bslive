use crate::internal::StartupEvent;
use crate::{StartupError, StartupEventDTO};
use bsnext_input::InputError;
use bsnext_output::OutputWriterTrait;
use std::io::Write;

impl OutputWriterTrait for StartupEvent {
    fn write_json<W: Write>(&self, sink: &mut W) -> anyhow::Result<()> {
        let as_dto = StartupEventDTO::from(self);
        writeln!(sink, "{}", serde_json::to_string(&as_dto)?)
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    }

    fn write_pretty<W: Write>(&self, sink: &mut W) -> anyhow::Result<()> {
        match self {
            StartupEvent::Started => {
                writeln!(sink, "started...")?;
            }
            StartupEvent::FailedStartup(err) => {
                writeln!(sink, "An error prevented startup!",)?;
                writeln!(sink)?;
                match err {
                    StartupError::InputError(InputError::BsLiveRules(bs_rules)) => {
                        let n = miette::GraphicalReportHandler::new();
                        let mut inner = String::new();
                        n.render_report(&mut inner, bs_rules).expect("write?");
                        writeln!(sink, "{}", inner)?;
                    }
                    StartupError::InputError(err) => {
                        writeln!(sink, "{}", err)?;
                    }
                    StartupError::Other(e) => {
                        writeln!(sink, "{}", e)?;
                    }
                    StartupError::ServerError(e) => {
                        writeln!(sink, "{}", e)?;
                    }
                }
            }
        }
        Ok(())
    }
}
