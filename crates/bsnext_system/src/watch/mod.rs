use crate::watch::watch_sub_opts::WatchSubOpts;
use bsnext_core::shared_args::LoggingOpts;
use bsnext_input::route::MultiWatch;
use bsnext_tracing::OutputFormat;
use watch_runner::WatchRunnerStr;

pub mod watch_runner;
pub mod watch_sub_opts;

#[derive(Debug, Default, Clone, clap::Parser)]
pub struct WatchCommand {
    /// Paths to watch
    #[arg(required = true)]
    pub paths: Vec<String>,
    #[arg(long, num_args(0..))]
    pub before: Vec<WatchRunnerStr>,
    /// sh Commands to run when files have changed
    #[arg(long, num_args(0..))]
    pub run: Vec<WatchRunnerStr>,
    /// if true, listed commands will execute once before watching starts
    #[arg(long)]
    pub initial: bool,
    /// provide this flag to disable command prefixes
    #[arg(long = "no-prefix", default_value = "false")]
    pub no_prefix: bool,
    /// logging options
    #[clap(flatten)]
    pub logging: LoggingOpts,
    /// output format
    #[arg(short, long, value_enum, default_value_t)]
    pub format: OutputFormat,
}

impl From<WatchCommand> for MultiWatch {
    fn from(value: WatchCommand) -> Self {
        let sub_opts = WatchSubOpts {
            paths: value.paths,
            run: value.run,
            before: value.before,
            initial: value.initial,
        };
        MultiWatch::from(sub_opts)
    }
}
