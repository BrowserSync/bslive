use crate::{OutputFormat, WriteOption};
use std::fs::File;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::Layer;

pub fn init_tracing_subscriber(debug_str: &str, format: OutputFormat, write_option: WriteOption) {
    let filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| debug_str.into());

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
                .with_file(true)
                .boxed()
        }
        (OutputFormat::Normal, WriteOption::File) | (OutputFormat::Tui, WriteOption::File) => {
            let file = File::create("bslive.log").expect("create bslive.log");
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(file)
                .with_thread_names(true)
                .with_line_number(true)
                .with_target(false)
                .with_file(true)
                .boxed()
        }
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();
}
