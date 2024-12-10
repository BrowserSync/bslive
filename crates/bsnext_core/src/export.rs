use std::path::PathBuf;

#[derive(Debug, clap::Parser)]
pub struct ExportCommand {
    /// The folder to export the files to. For current, provide '.'
    #[arg(long = "dir")]
    pub out_dir: PathBuf,
    /// When provided, just prints what might happen instead of actually causing side effects
    #[arg(long)]
    pub dry_run: bool,
}
