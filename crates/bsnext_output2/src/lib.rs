use std::fmt::{Display, Formatter};
use std::io::{Stderr, Stdout, Write};

#[derive(Debug, Default)]
pub enum OutputWriters {
    #[default]
    Pretty,
    Json,
}

impl Display for OutputWriters {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputWriters::Pretty => write!(f, "OutputWriters::Pretty"),
            OutputWriters::Json => write!(f, "OutputWriters::Json"),
        }
    }
}

pub trait OutputWriterTrait {
    fn write_json<W: Write>(&self, _sink: &mut W) -> anyhow::Result<()>;
    fn write_pretty<W: Write>(&self, _sink: &mut W) -> anyhow::Result<()>;
}

impl OutputWriters {
    pub fn write_evt<W: Write>(
        &self,
        evt: impl OutputWriterTrait,
        sink: &mut W,
    ) -> anyhow::Result<()> {
        match self {
            OutputWriters::Pretty => evt.write_pretty(sink),
            OutputWriters::Json => evt.write_json(sink),
        }
    }
}

#[non_exhaustive]
#[derive(Debug)]
pub enum OutputTarget<'a> {
    Stdout(StdoutTarget<'a>),
}

#[derive(Debug)]
pub struct StdoutTarget<'a> {
    stdout: &'a Stdout,
    stderr: &'a Stderr,
}

impl<'a> StdoutTarget<'a> {
    pub fn new(out: &'a Stdout, err: &'a Stderr) -> Self {
        Self {
            stdout: out,
            stderr: err,
        }
    }

    pub fn close(&mut self) {
        match (self.stderr.flush(), self.stdout.flush()) {
            (Ok(_), Ok(_)) => {}
            _ => tracing::error!("could not flush"),
        };
    }

    pub fn output(&mut self) -> impl Write + use<'a> {
        self.stdout
    }

    pub fn error(&mut self) -> impl Write + use<'a> {
        self.stderr
    }
}
