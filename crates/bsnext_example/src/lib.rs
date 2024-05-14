use crate::basic::BasicExample;
use crate::lit::LitExample;
use crate::md::MdExample;
use bsnext_input::server_config::Identity;
use bsnext_input::Input;

pub mod basic;
pub mod lit;
pub mod md;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum Example {
    Basic,
    Lit,
    Md,
}

impl Example {
    pub fn into_input(self, identity: Identity) -> Input {
        match self {
            Example::Basic => BasicExample.into_input(Some(identity)),
            Example::Lit => LitExample.into_input(Some(identity)),
            Example::Md => MdExample.into_input(Some(identity)),
        }
    }
}
