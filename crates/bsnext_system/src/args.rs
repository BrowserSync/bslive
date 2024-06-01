use crate::Example;
use bsnext_input::target::TargetKind;
use bsnext_tracing::{LogLevel, OutputFormat};

#[derive(clap::Parser, Debug)]
pub struct Args {
    #[arg(short, long, value_enum)]
    pub log_level: Option<LogLevel>,

    #[arg(long)]
    pub otel: bool,

    /// output internal logs to bslive.log in the current directory
    #[arg(long, name = "write-log")]
    pub write_log: bool,

    #[arg(short, long, value_enum)]
    pub format: Option<OutputFormat>,

    /// Input files
    #[arg(short, long)]
    pub input: Vec<String>,

    /// Write input to disk
    #[arg(long)]
    pub write: bool,

    /// Write input to disk
    #[arg(long, requires = "write")]
    pub target: Option<TargetKind>,

    #[arg(long, value_enum)]
    pub example: Option<Example>,

    /// create a temp folder for examples instead of using the current dir
    #[arg(long, requires = "example")]
    pub temp: bool,

    /// create a temp folder for examples instead of using the current dir
    #[arg(long, requires = "example")]
    pub name: Option<String>,

    /// Only works with `--example` - specify a port instead of a random one    
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Paths to serve + possibly watch, incompatible with `-i` option
    pub paths: Vec<String>,
}
