use crate::args::Args;
use crate::start_kind::start_from_example::StartFromExample;
use crate::start_kind::start_from_inputs::{StartFromInput, StartFromInputPaths};
use crate::start_kind::start_from_paths::StartFromDirPaths;
use bsnext_fs_helpers::{fs_write_str, FsWriteError, WriteMode};
use bsnext_input::route::{CorsOpts, Opts};
use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
use bsnext_input::target::TargetKind;
use bsnext_input::InputWriter;
use bsnext_input::{Input, InputError};
use std::path::{Path, PathBuf};

pub mod start_from_example;
pub mod start_from_inputs;
pub mod start_from_paths;

#[derive(Debug)]
pub enum StartKind {
    FromInput(StartFromInput),
    FromInputPaths(StartFromInputPaths),
    FromExample(StartFromExample),
    FromDirPaths(StartFromDirPaths),
}

impl StartKind {
    pub fn from_args(args: &Args) -> Self {
        if let Some(example) = args.example {
            return StartKind::FromExample(StartFromExample {
                example,
                write_input: args.write,
                port: args.port,
                temp: args.temp,
                name: args.name.clone(),
                target_kind: args
                    .target
                    .as_ref()
                    .map(ToOwned::to_owned)
                    .unwrap_or_default(),
                dir: args.dir.clone(),
                force: args.force,
            });
        }

        if !args.paths.is_empty() {
            tracing::info!("cors arg {}", args.cors);
            StartKind::FromDirPaths(StartFromDirPaths {
                paths: args.paths.clone(),
                write_input: args.write,
                port: args.port,
                force: args.force,
                route_opts: Opts {
                    cors: args.cors.then_some(CorsOpts::Cors(true)),
                    ..Default::default()
                },
            })
        } else {
            StartKind::FromInputPaths(StartFromInputPaths {
                input_paths: args.input.clone(),
                port: args.port,
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
            StartKind::FromDirPaths(from_dir_paths) => from_dir_paths.input(ctx),
            StartKind::FromInput(from_inputs) => from_inputs.input(ctx),
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
