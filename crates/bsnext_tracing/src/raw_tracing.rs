use crate::{LineNumberOption, OutputFormat, WriteOption};
use std::fs::File;
use tracing_subscriber::layer::Layered;
use tracing_subscriber::{EnvFilter, Layer, Registry};

pub fn create_filter_and_fmt(
    debug_str: &str,
    format: OutputFormat,
    write_option: WriteOption,
    line_opts: LineNumberOption,
) -> (
    EnvFilter,
    Box<dyn Layer<Layered<EnvFilter, Registry>> + Send + Sync>,
) {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| debug_str.into());
    let include_lines = line_opts == LineNumberOption::FileAndLineNumber;

    let fmt_layer = match (format, write_option) {
        (OutputFormat::Json, WriteOption::None) => tracing_subscriber::fmt::layer()
            .without_time()
            .json()
            .with_file(false)
            .boxed(),
        (OutputFormat::Json, WriteOption::File) => {
            let file = File::create("bslive.log").expect("create bslive.log");
            tracing_subscriber::fmt::layer()
                .json()
                .with_ansi(false)
                // todo(alpha): use this example as a way to move this output into the terminal window
                .with_writer(file)
                .boxed()
        }
        (OutputFormat::Normal, WriteOption::None) | (OutputFormat::Tui, WriteOption::None) => {
            tracing_subscriber::fmt::layer()
                .without_time()
                .with_ansi(true)
                .with_target(false)
                .with_file(include_lines)
                .with_line_number(include_lines)
                .boxed()
        }
        (OutputFormat::Normal, WriteOption::File) | (OutputFormat::Tui, WriteOption::File) => {
            let file = File::create("bslive.log").expect("create bslive.log");
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(file)
                .with_target(false)
                .with_file(include_lines)
                .with_line_number(include_lines)
                .boxed()
        }
    };

    (filter, fmt_layer)
}
