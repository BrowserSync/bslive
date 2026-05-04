use crate::start::start_kind::run_from_input::RunFromInputPaths;
use crate::start::start_kind::start_from_inputs::{StartFromInput, StartFromInputPaths};
use crate::start::start_kind::start_from_paths::StartFromPaths;
use bsnext_fs_helpers::{fs_write_str, FsWriteError, WriteMode};
use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
use bsnext_input::target::TargetKind;
use bsnext_input::InputWriter;
use bsnext_input::{Input, InputError};
use std::path::{Path, PathBuf};

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

pub fn fs_write_input(
    cwd: &Path,
    input: &Input,
    target_kind: TargetKind,
    write_mode: &WriteMode,
) -> Result<PathBuf, FsWriteError> {
    let string = match target_kind {
        TargetKind::Yaml => bsnext_yaml::yaml_writer::YamlWriter.input_to_str(input),
        TargetKind::Toml => todo!("toml missing"),
        TargetKind::Md => bsnext_md::md_writer::MdWriter.input_to_str(input),
        TargetKind::Html => bsnext_html::html_writer::HtmlWriter.input_to_str(input),
    };
    let name = match target_kind {
        TargetKind::Yaml => "bslive.yml",
        TargetKind::Toml => todo!("toml missing"),
        TargetKind::Md => "bslive.md",
        TargetKind::Html => "bslive.html",
    };

    fs_write_str(cwd, &PathBuf::from(name), &string, write_mode)
}
