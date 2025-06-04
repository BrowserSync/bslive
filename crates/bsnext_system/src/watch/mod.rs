#[derive(Debug, Clone, clap::Parser)]
pub struct WatchCommand {
    /// Paths to watch
    pub trailing: Vec<String>,
    /// Command to run when files have changed
    #[arg(long = "command", short)]
    pub command: Vec<String>,
    /// Initial command to run before starting watchers
    #[arg(long = "initial", short)]
    pub initial: Vec<String>,
}
