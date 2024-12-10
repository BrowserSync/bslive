use crate::Example;
use bsnext_core::export::ExportCommand;
use bsnext_input::target::TargetKind;
use bsnext_tracing::{LogLevel, OutputFormat};
// bslive route --path=/ --dir=

#[derive(clap::Parser, Debug)]
#[command(version, name = "Browsersync Live")]
pub struct Args {
    #[arg(short, long, value_enum)]
    pub log_level: Option<LogLevel>,

    #[arg(long)]
    pub otel: bool,

    /// output internal logs to bslive.log in the current directory
    #[arg(long, name = "write-log")]
    pub write_log: bool,

    #[arg(short, long, value_enum, default_value_t)]
    pub format: OutputFormat,

    /// Input files
    #[arg(short, long)]
    pub input: Vec<String>,

    /// Write input to disk
    #[arg(long)]
    pub write: bool,

    /// Force write over directories or files (dangerous)
    #[arg(long, requires = "write")]
    pub force: bool,

    /// Write input to disk
    #[arg(long, requires = "write")]
    pub target: Option<TargetKind>,

    #[arg(long, value_enum)]
    pub example: Option<Example>,

    /// create a temp folder for examples instead of using the current dir
    #[arg(long, requires = "example")]
    pub temp: bool,

    /// Override output folder (not compatible with 'temp')
    #[arg(long, requires = "example", conflicts_with = "temp")]
    pub dir: Option<String>,

    /// create a temp folder for examples instead of using the current dir
    #[arg(long, requires = "example", conflicts_with = "dir")]
    pub name: Option<String>,

    /// Only works with `--example` - specify a port instead of a random one
    #[arg(short, long)]
    pub port: Option<u16>,

    #[command(subcommand)]
    pub command: Option<SubCommands>,

    /// Paths to serve + possibly watch, incompatible with `-i` option
    pub paths: Vec<String>,
}

#[derive(Debug, clap::Subcommand)]
pub enum SubCommands {
    /// Export raw entries to files
    Export(ExportCommand),
}
