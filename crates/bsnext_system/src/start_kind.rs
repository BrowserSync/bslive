use crate::args::Args;
use crate::start_kind::start_from_example::StartFromExample;
use crate::start_kind::start_from_inputs::{StartFromInput, StartFromInputPaths};
use crate::start_kind::start_from_paths::StartFromPaths;
use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
use std::fs;
use std::path::{Path, PathBuf};

use bsnext_input::target::TargetKind;
use bsnext_input::{Input, InputError, InputWriteError};

pub mod start_from_example;
pub mod start_from_inputs;
pub mod start_from_paths;

#[derive(Debug)]
pub enum StartKind {
    FromInput(StartFromInput),
    FromInputPaths(StartFromInputPaths),
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
            StartKind::FromInputPaths(StartFromInputPaths {
                input_paths: args.input.clone(),
            })
        }
    }
    pub fn from_input(input: Input) -> Self {
        Self::FromInput(StartFromInput { input })
    }
}

impl SystemStart for StartKind {
    fn input(&self, ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        match self {
            StartKind::FromInputPaths(from_inputs) => from_inputs.input(ctx),
            StartKind::FromExample(from_example) => from_example.input(ctx),
            StartKind::FromPaths(from_input_paths) => from_input_paths.input(ctx),
            StartKind::FromInput(from_inputs) => from_inputs.input(ctx),
        }
    }
}

pub fn fs_write_input(
    cwd: &Path,
    input: &Input,
    target_kind: TargetKind,
) -> Result<PathBuf, InputWriteError> {
    let string = match target_kind {
        TargetKind::Yaml => bsnext_yaml::input_to_str(input),
        TargetKind::Toml => todo!("toml missing"),
        TargetKind::Md => bsnext_md::input_to_str(input),
    };
    let name = match target_kind {
        TargetKind::Yaml => "bslive.yml",
        TargetKind::Toml => todo!("toml missing"),
        TargetKind::Md => "bslive.md",
    };
    let next_path = cwd.join(name);
    fs::write(&next_path, string)
        .map(|()| next_path.clone())
        .map_err(|_e| InputWriteError::FailedWrite { path: next_path })
}
