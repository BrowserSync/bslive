// pub mod ratatui;

use std::fmt::{Display, Formatter};
use std::io::Write;
use stdout::StdoutTarget;

pub mod stdout;

#[derive(Debug, Hash, Default, Clone)]
pub enum OutputGlobalPrefix {
    #[default]
    NodePrefix,
    None,
}

#[derive(Debug)]
pub enum OutputWriters {
    Pretty { prefix: OutputGlobalPrefix },
    Json,
}

impl Default for OutputWriters {
    fn default() -> Self {
        Self::Pretty {
            prefix: OutputGlobalPrefix::NodePrefix,
        }
    }
}

impl Display for OutputWriters {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputWriters::Pretty { .. } => write!(f, "OutputWriters::Pretty"),
            OutputWriters::Json => write!(f, "OutputWriters::Json"),
        }
    }
}

pub trait OutputWriterTrait: std::fmt::Debug {
    fn write_json<W: Write>(&self, _sink: &mut W) -> anyhow::Result<()>;
    fn write_pretty<W: Write>(
        &self,
        prefix: &OutputGlobalPrefix,
        _sink: &mut W,
    ) -> anyhow::Result<()>;
}

impl OutputWriters {
    pub fn write_evt<W: Write>(
        &self,
        evt: &impl OutputWriterTrait,
        sink: &mut W,
    ) -> anyhow::Result<()> {
        match self {
            OutputWriters::Pretty { prefix } => evt.write_pretty(prefix, sink),
            OutputWriters::Json => evt.write_json(sink),
        }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub enum OutputTarget<'a> {
    Stdout(StdoutTarget<'a>),
}
