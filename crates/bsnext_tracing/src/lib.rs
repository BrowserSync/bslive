pub mod otel_cli;
pub mod otlp;
pub mod raw_tracing;

use crate::raw_tracing::create_filter_and_fmt;
// use bsnext_otel::OtelGuard;
use crate::otel_cli::OtelGuard;
use std::fmt::{Display, Formatter};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    clap::ValueEnum,
    serde::Serialize,
    serde::Deserialize,
)]
#[serde(rename_all = "lowercase")]
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

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum OutputFormat {
    Json,
    #[default]
    Normal,
    Tui,
}

impl Display for OutputFormat {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Normal => write!(f, "normal"),
            OutputFormat::Tui => write!(f, "tui"),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum WriteOption {
    File,
    None,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum LineNumberOption {
    FileAndLineNumber,
    None,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OtelOption {
    On,
    Off,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum LogHttp {
    On,
    #[default]
    Off,
}

#[derive(Debug)]
pub enum TracingGuard {
    None,
    OtelGuard(OtelGuard),
}

pub fn init_tracing(
    log_level: Option<LogLevel>,
    log_http: LogHttp,
    format: OutputFormat,
    write_option: WriteOption,
    line_opts: LineNumberOption,
) -> TracingGuard {
    let level = level(log_level, log_http);

    let (filter, fmt_layer) =
        raw_tracing::create_filter_and_fmt(&level, format, write_option, line_opts);

    let _registry = tracing_subscriber::registry()
        .with(filter)
        .with(fmt_layer)
        .init();

    TracingGuard::None
}

pub fn init_tracing_with_otel(
    log_level: Option<LogLevel>,
    log_http: LogHttp,
    format: OutputFormat,
    write_option: WriteOption,
    line_opts: LineNumberOption,
) -> TracingGuard {
    let level = level(log_level, log_http);

    let (filter, fmt_layer) = create_filter_and_fmt(&level, format, write_option, line_opts);
    let prov = otel_cli::init_tracing_subscriber((filter, fmt_layer));

    TracingGuard::OtelGuard(prov)
}

pub fn level(log_level: Option<LogLevel>, log_http: LogHttp) -> String {
    match (log_level, log_http) {
        (None, LogHttp::Off) => String::new(),
        (None, LogHttp::On) => "tower_http=debug".to_string(),
        (Some(level), LogHttp::On) => {
            let level = level.to_string();
            let lines = [
                format!("bsnext={level}"),
                "bsnext_fs::stream=info".to_string(),
                "bsnext_fs::buffered_debounce=info".to_string(),
                "tower_http=debug".to_string(),
                // "bsnext_fs::watcher=info".to_string(),
                // "bsnext_core::server_actor=info".to_string(),
            ];

            lines.join(",")
        }
        (Some(level), LogHttp::Off) => {
            let level = level.to_string();
            let lines = [
                format!("bsnext={level}"),
                "bsnext_fs::stream=info".to_string(),
                "bsnext_fs::buffered_debounce=info".to_string(),
                // "bsnext_fs::watcher=info".to_string(),
                // "bsnext_core::server_actor=info".to_string(),
            ];

            lines.join(",")
        }
    }
}
