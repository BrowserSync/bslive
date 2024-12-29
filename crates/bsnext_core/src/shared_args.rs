#[derive(Debug, Clone, clap::Parser)]
pub struct LoggingOpts {
    #[arg(short, long, value_enum)]
    pub log_level: Option<bsnext_tracing::LogLevel>,

    #[arg(long)]
    pub otel: bool,

    /// output internal logs to bslive.log in the current directory
    #[arg(long, name = "write-log")]
    pub write_log: bool,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct FsOpts {
    /// Write input to disk
    #[arg(long)]
    pub write: bool,

    /// Force write over directories or files (dangerous)
    #[arg(long, requires = "write")]
    pub force: bool,
}

#[derive(Debug, Clone, clap::Parser)]
pub struct InputOpts {
    /// Provide a path to an input file
    #[arg(short, long)]
    pub input: Vec<String>,
}
