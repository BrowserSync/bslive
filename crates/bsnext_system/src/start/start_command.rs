#[derive(Debug, Clone, clap::Parser)]
pub struct StartCommand {
    /// Should permissive cors headers be added to all responses?
    #[arg(long)]
    pub cors: bool,

    /// Specify a port instead of a random one
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Paths to serve + possibly watch, incompatible with `-i` option
    pub trailing: Vec<String>,
}
