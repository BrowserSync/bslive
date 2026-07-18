use crate::archy::{archy, Prefix};
use crate::internal::{InternalEvents, InternalEventsDTO, TaskAction, TaskActionStage};
use crate::InputErrorDTO;
use bsnext_input::InputError;
use bsnext_output::OutputWriterTrait;
use std::io::Write;

impl OutputWriterTrait for InternalEvents {
    fn write_json<W: Write>(&self, sink: &mut W) -> anyhow::Result<()> {
        match self {
            InternalEvents::InputError(err) => {
                let e = InputErrorDTO::from(err);
                writeln!(sink, "{}", serde_json::to_string(&e)?)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            }
            InternalEvents::StartupError(startup) => {
                let str = startup.to_string();
                let json = serde_json::json!({
                    "_unstable_InternalEvents::StartupError": str
                });
                writeln!(sink, "{}", serde_json::to_string(&json)?)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            }
            InternalEvents::TaskAction(TaskAction { stage: action, .. }) => match action {
                TaskActionStage::Started { tree: _ } => {
                    // writeln!(sink, "{}", serde_json::to_string(&action)?)
                    //     .map_err(|e| anyhow::anyhow!(e.to_string()))?;
                }
                TaskActionStage::Ended { report: _, .. } => {
                    // let s = archy(tree, Prefix::None);
                    // write!(sink, "{s}")?;
                    // let as_json = InternalEventsDTO::TaskReport {
                    //     id: report.id().to_string(),
                    // };
                    // writeln!(sink, "{}", serde_json::to_string(&as_json)?)
                    //     .map_err(|e| anyhow::anyhow!(e.to_string()))?;
                }
                TaskActionStage::Error => {}
            },
            InternalEvents::TaskSpecDisplay { tree } => {
                let evt = InternalEventsDTO::TaskTreeDisplay { tree: tree.clone() };
                writeln!(sink, "{}", serde_json::to_string(&evt)?)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            }
        }
        Ok(())
    }

    fn write_pretty<W: Write>(&self, sink: &mut W) -> anyhow::Result<()> {
        match self {
            InternalEvents::InputError(InputError::BsLiveRules(bs_rules)) => {
                let n = miette::GraphicalReportHandler::new();
                let mut inner = String::new();
                n.render_report(&mut inner, bs_rules).expect("write?");
                writeln!(sink, "{inner}")?;
            }
            InternalEvents::InputError(err) => {
                writeln!(sink, "{err}")?;
            }
            InternalEvents::StartupError(err) => {
                writeln!(sink, "{err}")?;
            }
            InternalEvents::TaskAction(..) => {
                // no-op
            }
            InternalEvents::TaskSpecDisplay { tree } => {
                let s = archy(tree, Prefix::None);
                write!(sink, "{s}")?;
            }
        }
        Ok(())
    }
}
