use crate::start::start_kind::run_from_input::RunFromInputPaths;
use crate::start::start_kind::start_from_inputs::{StartFromInput, StartFromInputPaths};
use crate::start::start_kind::start_from_paths::StartFromPaths;
use crate::start::SystemStart;
use bsnext_input::startup::{StartupContext, SystemStartArgs};
use bsnext_input::InputError;

pub mod run_from_input;
pub mod start_from_inputs;
pub mod start_from_paths;

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum StartKind {
    Run(RunFromInputPaths),
    FromInput(StartFromInput),
    FromInputPaths(StartFromInputPaths),
    FromPaths(StartFromPaths),
}

impl SystemStart for StartKind {
    fn resolve_input(&self, ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        match self {
            StartKind::FromInputPaths(from_inputs) => from_inputs.resolve_input(ctx),
            StartKind::FromPaths(from_trailing_args) => from_trailing_args.resolve_input(ctx),
            StartKind::FromInput(from_inputs) => from_inputs.resolve_input(ctx),
            StartKind::Run(run_from_input) => run_from_input.resolve_input(ctx),
        }
    }
}
