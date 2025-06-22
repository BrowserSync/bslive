use bsnext_core::shared_args::LoggingOpts;
use bsnext_tracing::OutputFormat;

#[derive(Debug, Default, Clone, clap::Parser)]
pub struct StartCommand {
    /// Should permissive cors headers be added to all responses?
    #[arg(long)]
    pub cors: bool,

    /// Specify a port instead of a random one
    #[arg(short, long)]
    pub port: Option<u16>,

    #[arg(long = "proxy")]
    pub proxies: Vec<String>,

    /// logging options
    #[clap(flatten)]
    pub logging: LoggingOpts,

    /// output options
    #[arg(short, long, value_enum, default_value_t)]
    pub format: OutputFormat,

    /// Paths to serve + possibly watch, incompatible with `-i` option
    pub trailing: Vec<String>,
}
