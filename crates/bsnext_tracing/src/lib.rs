mod otlp;

use std::fmt::{Display, Formatter};

pub use crate::otlp::{init_tracing_subscriber, OtelGuard};

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
pub enum OtelOption {
    On,
    Off,
}
pub fn init_tracing(
    log_level: Option<LogLevel>,
    format: OutputFormat,
    write_option: WriteOption,
    otel: OtelOption,
) -> OtelGuard {
    match log_level {
        None => OtelGuard {
            meter_provider: None,
        },
        Some(level) => {
            let level = level.to_string();
            let lines = [
                format!("bsnext={level}"),
                format!("bsnext_core={level}"),
                format!("tower_http={level}"),
                "bsnext_fs::stream=info".to_string(),
                "bsnext_fs::watcher=info".to_string(),
                "bsnext_fs::buffered_debounce=info".to_string(),
                // "bsnext_core::server_actor=info".to_string(),
            ];
            let debug_str = lines.join(",");
            init_tracing_subscriber(&debug_str, format, write_option, otel)
        }
    }
}
