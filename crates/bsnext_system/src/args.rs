use crate::start::start_command::StartCommand;
use crate::watch::WatchCommand;
use bsnext_core::export::ExportCommand;
use bsnext_core::shared_args::{FsOpts, InputOpts, LoggingOpts};
use bsnext_example::ExampleCommand;
use bsnext_tracing::OutputFormat;
// bslive route --path=/ --dir=

#[derive(clap::Parser, Clone, Debug)]
#[command(version, name = "Browsersync Live", propagate_version = true)]
pub struct Args {
    #[clap(flatten)]
    pub logging: LoggingOpts,

    #[arg(short, long, value_enum, default_value_t)]
    pub format: OutputFormat,

    #[clap(flatten)]
    pub input_opts: InputOpts,

    #[clap(flatten)]
    pub fs_opts: FsOpts,

    /// Only used if we're going to fallback
    #[arg(short, long)]
    pub port: Option<u16>,

    #[command(subcommand)]
    pub command: Option<SubCommands>,

    /// Paths to serve + possibly watch, incompatible with `-i` option
    pub trailing: Vec<String>,
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum SubCommands {
    /// Start the services
    Start(StartCommand),
    /// Export raw entries to files
    Export(ExportCommand),
    /// Run an example project
    Example(ExampleCommand),
    /// Just use file watching
    Watch(WatchCommand),
}
