use crate::run::RunCommand;
use crate::start::start_command::StartCommand;
use crate::watch::watch_sub_opts::WatchSubOpts;
use crate::watch::WatchCommand;
use bsnext_core::shared_args::{InputOpts, LoggingOpts};
use bsnext_output::OutputGlobalPrefix;
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

    /// Only used if we're going to fallback
    #[arg(short, long)]
    pub port: Option<u16>,

    #[command(subcommand)]
    pub command: Option<SubCommands>,

    #[clap(flatten)]
    pub watch_opts: WatchSubOpts,

    /// pass '--no-watch' to prevent auto watchers. can't be used with 'input'
    #[clap(long)]
    pub no_watch: bool,

    /// pass '--no-prefix' to prevent output lines being prefixed with the node tree path by default'
    #[clap(long)]
    pub no_prefix: bool,

    /// Paths to serve + possibly watch, incompatible with `-i` option
    pub trailing: Vec<String>,
}

impl Args {
    /// Chose the given command or just default to 'start'
    pub(crate) fn command(self) -> (SubCommands, InputOpts) {
        let logging = *self.logging();
        let format = self.format();
        self.command.map_or_else(
            || {
                let default_cmd = SubCommands::Start(StartCommand {
                    cors: false,
                    port: self.port,
                    trailing: self.trailing.clone(),
                    proxies: vec![],
                    watch_sub_opts: self.watch_opts,
                    logging,
                    format,
                    no_watch: self.no_watch,
                    no_prefix: self.no_prefix,
                });
                (default_cmd, self.input_opts.clone())
            },
            |cmd| (cmd, self.input_opts.clone()),
        )
    }

    pub fn global_prefix(&self) -> OutputGlobalPrefix {
        if self.no_prefix {
            return OutputGlobalPrefix::None;
        }
        self.command
            .as_ref()
            .and_then(|cmd| {
                let global_refused = match cmd {
                    SubCommands::Start(start) => start.no_prefix,
                    SubCommands::Watch(watch) => watch.no_prefix,
                    SubCommands::Run(run) => run.no_prefix,
                };
                global_refused.then_some(OutputGlobalPrefix::None)
            })
            .unwrap_or_default()
    }
}

impl Args {
    pub fn logging(&self) -> &LoggingOpts {
        match &self.command {
            Some(SubCommands::Watch(WatchCommand { logging, .. })) => logging,
            Some(SubCommands::Start(StartCommand { logging, .. })) => logging,
            Some(SubCommands::Run(RunCommand { logging, .. })) => logging,
            _ => &self.logging,
        }
    }
    pub fn format(&self) -> OutputFormat {
        match &self.command {
            Some(SubCommands::Watch(WatchCommand { format, .. })) => format.to_owned(),
            Some(SubCommands::Start(StartCommand { format, .. })) => format.to_owned(),
            Some(SubCommands::Run(RunCommand { format, .. })) => format.to_owned(),
            _ => self.format,
        }
    }
}

#[derive(Debug, Clone, clap::Subcommand)]
pub enum SubCommands {
    Start(StartCommand),
    Watch(WatchCommand),
    Run(RunCommand),
}
