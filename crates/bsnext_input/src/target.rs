#[derive(Debug, Clone, Default, clap::ValueEnum)]
pub enum TargetKind {
    #[default]
    Yaml,
    Toml,
    Md,
    Html,
}
