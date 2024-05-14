use crate::args::Args;
use crate::start_kind::start_from_example::StartFromExample;
use crate::start_kind::start_from_inputs::StartFromInputPaths;
use crate::start_kind::start_from_paths::StartFromPaths;
use crate::startup::{StartupContext, SystemStart};

use bsnext_input::{Input, InputError};
use std::path::PathBuf;

pub mod start_from_example;
pub mod start_from_inputs;
pub mod start_from_paths;

#[derive(Debug)]
pub enum StartKind {
    FromInputs(StartFromInputPaths),
    FromExample(StartFromExample),
    FromPaths(StartFromPaths),
}

impl StartKind {
    pub fn from_args(args: Args) -> Self {
        if let Some(example) = args.example {
            return StartKind::FromExample(StartFromExample {
                example,
                write_input: args.write,
                port: args.port,
                temp: args.temp,
                name: args.name,
                target_kind: args.target.unwrap_or_default(),
            });
        }

        if !args.paths.is_empty() {
            StartKind::FromPaths(StartFromPaths {
                paths: args.paths,
                write_input: args.write,
                port: args.port,
            })
        } else {
            StartKind::FromInputs(StartFromInputPaths {
                input_paths: args.input.clone(),
            })
        }
    }
}

impl SystemStart for StartKind {
    fn input(&self, ctx: &StartupContext) -> Result<(Input, Option<PathBuf>), InputError> {
        match self {
            Self::FromInputs(from_inputs) => from_inputs.input(ctx),
            Self::FromExample(from_example) => from_example.input(ctx),
            Self::FromPaths(from_paths) => from_paths.input(ctx),
        }
    }
}
