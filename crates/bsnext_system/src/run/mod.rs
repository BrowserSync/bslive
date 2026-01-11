use bsnext_core::shared_args::LoggingOpts;
use bsnext_tracing::OutputFormat;

#[derive(Debug, Clone, clap::Parser)]
pub struct RunCommand {
    /// commands to run
    pub sh_commands_implicit: Vec<String>,
    /// commands to run
    #[arg(long = "sh")]
    pub sh_commands: Vec<String>,
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