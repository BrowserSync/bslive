use crate::basic::BasicExample;
use crate::lit::LitExample;
use crate::md::MdExample;
use crate::playground::PlaygroundExample;
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::{InputSource, InputSourceKind};

pub mod basic;
pub mod lit;
pub mod md;

pub mod playground;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum Example {
    Basic,
    Lit,
    Md,
    #[default]
    Playground,
}

use bsnext_input::target::TargetKind;

#[derive(Debug, Clone, clap::Parser)]
pub struct ExampleCommand {
    /// Write input to disk
    #[arg(long)]
    pub target: Option<TargetKind>,

    #[arg(long, value_enum, default_value_t)]
    pub example: Example,

    /// create a temp folder for examples instead of using the current dir
    #[arg(long, requires = "example")]
    pub temp: bool,

    /// Override output folder (not compatible with 'temp')
    #[arg(long, requires = "example", conflicts_with = "temp")]
    pub dir: Option<String>,

    /// create a temp folder for examples instead of using the current dir
    #[arg(long, requires = "example", conflicts_with = "dir")]
    pub name: Option<String>,
}

impl InputSource for Example {
    fn into_input(self, identity: Option<ServerIdentity>) -> InputSourceKind {
        match self {
            Example::Basic => BasicExample.into_input(identity),
            Example::Lit => LitExample.into_input(identity),
            Example::Md => MdExample.into_input(identity),
            Example::Playground => PlaygroundExample.into_input(identity),
        }
    }
}
