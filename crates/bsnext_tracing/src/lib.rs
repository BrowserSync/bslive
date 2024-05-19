use std::fmt::{Display, Formatter};
use std::fs::File;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum LogLevel {
    Info,
    Debug,
    Trace,
    Error,
}

impl Display for LogLevel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Info => write!(f, "info"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum OutputFormat {
    Json,
    Normal,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum WriteOption {
    File,
    None,
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Normal => write!(f, "normal"),
        }
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        Self::Normal
    }
}
pub fn init_tracing(
    log_level: Option<LogLevel>,
    format: Option<OutputFormat>,
    write_option: WriteOption,
) {
    let log_level = log_level.unwrap_or(LogLevel::Error);
    let level = log_level.to_string();
    let lines = [
        format!("bsnext={level}"),
        format!("bsnext_core={level}"),
        "bsnext_fs::stream=info".to_string(),
        "bsnext_fs::watcher=info".to_string(),
        // "bsnext_core::server_actor=info".to_string(),
    ];
    let debug_str = lines.join(",");

    match (format.unwrap_or_default(), write_option) {
        (OutputFormat::Json, WriteOption::None) => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .without_time()
                .json()
                .with_file(false);
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| debug_str.into()),
                )
                .with(fmt_layer)
                .init();
        }
        (OutputFormat::Json, WriteOption::File) => {
            let file = File::create("out.log").expect("create out.log");
            let fmt_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_ansi(false)
                .with_writer(file);
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| debug_str.into()),
                )
                .with(fmt_layer)
                .init();
        }
        (OutputFormat::Normal, WriteOption::None) => {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .without_time()
                .with_file(false);
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| debug_str.into()),
                )
                .with(fmt_layer)
                .init();
        }
        (OutputFormat::Normal, WriteOption::File) => {
            let file = File::create("out.log").expect("create out.log");
            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(file);
            tracing_subscriber::registry()
                .with(
                    tracing_subscriber::EnvFilter::try_from_default_env()
                        .unwrap_or_else(|_| debug_str.into()),
                )
                .with(fmt_layer)
                .init();
        }
    };
}
