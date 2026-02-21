use crate::run::RunCommand;
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
    logging: LoggingOpts,

    #[arg(short, long, value_enum, default_value_t)]
    format: OutputFormat,

    #[clap(flatten)]
    pub input_opts: InputOpts,

    #[clap(flatten)]
    pub fs_opts: FsOpts,

    /// Only used if we're going to fallback
    #[arg(short, long)]
    pub port: Option<u16>,

    #[command(subcommand)]
    pub command: Option<SubCommands>,

    #[clap(flatten)]
    pub watch_opts: WatchOpts,

    /// Paths to serve + possibly watch, incompatible with `-i` option
    pub trailing: Vec<String>,
}

#[derive(Debug, Default, Clone, clap::Parser)]
pub struct WatchOpts {
    #[arg(long = "watch.paths", num_args(0..))]
    pub paths: Vec<String>,
    #[arg(long = "watch.command", num_args(0..))]
    pub command: Vec<String>,
}

impl From<WatchOpts> for WatchCommand {
    fn from(value: WatchOpts) -> Self {
        WatchCommand {
            paths: value.paths,
            command: value.command,
            initial: vec![],
            no_prefix: false,
            logging: Default::default(),
            format: Default::default(),
        }
    }
}

impl Args {
    pub fn logging(&self) -> &LoggingOpts {
        match &self.command {
            Some(SubCommands::Watch(WatchCommand { logging, .. })) => logging,
            Some(SubCommands::Start(StartCommand { logging, .. })) => logging,
            Some(SubCommands::Export(ExportCommand { logging, .. })) => logging,
            Some(SubCommands::Run(RunCommand { logging, .. })) => logging,
            _ => &self.logging,
        }
    }
    pub fn format(&self) -> OutputFormat {
        match &self.command {
            Some(SubCommands::Watch(WatchCommand { format, .. })) => format.to_owned(),
            Some(SubCommands::Start(StartCommand { format, .. })) => format.to_owned(),
            Some(SubCommands::Export(ExportCommand { format, .. })) => format.to_owned(),
            Some(SubCommands::Run(RunCommand { format, .. })) => format.to_owned(),
            _ => self.format,
        }
    }
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
    /// Just run tasks
    Run(RunCommand),
}
