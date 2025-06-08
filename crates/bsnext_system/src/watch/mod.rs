#[derive(Debug, Clone, clap::Parser)]
pub struct WatchCommand {
    /// Paths to watch
    #[arg(required = true)]
    pub paths: Vec<String>,
    /// Commands to run when files have changed
    #[arg(long = "command", short)]
    pub command: Vec<String>,
    /// Initial command to run before starting watchers
    #[arg(long = "initial", short)]
    pub initial: Vec<String>,
    /// provide this flag to disable command prefixes
    #[arg(long = "no-prefix", default_value = "false")]
    pub no_prefix: bool,
}
