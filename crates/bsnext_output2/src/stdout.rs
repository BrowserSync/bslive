use crate::{OutputWriterTrait, OutputWriters};
use std::io::{Stderr, Stdout, Write};

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

// Helper for the common task of printing a set of events or an error
pub fn completion_writer(
    writer: OutputWriters,
    result: Result<Vec<impl OutputWriterTrait>, impl OutputWriterTrait>,
) -> anyhow::Result<()> {
    let stdout = &mut std::io::stdout();
    let stderr = &mut std::io::stderr();
    let mut sink = StdoutTarget::new(stdout, stderr);

    match result {
        Ok(events) => {
            for export_event in events {
                writer.write_evt(export_event, &mut sink.output())?;
            }
            sink.close();
            Ok(())
        }
        Err(err) => {
            writer.write_evt(err, &mut sink.error())?;
            sink.close();
            Err(anyhow::anyhow!("export failed"))
        }
    }
}
