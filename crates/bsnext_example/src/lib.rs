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

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum Example {
    Basic,
    Lit,
    Md,
    Playground,
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
