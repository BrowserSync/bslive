use crate::start::start_command::StartCommand;
use crate::start::start_kind::start_from_example::StartFromExample;
use crate::start::start_kind::start_from_inputs::{StartFromInput, StartFromInputPaths};
use crate::start::start_kind::start_from_paths::StartFromTrailingArgs;
use bsnext_core::shared_args::{FsOpts, InputOpts};
use bsnext_fs_helpers::{fs_write_str, FsWriteError, WriteMode};
use bsnext_input::route::{CorsOpts, Opts, Route};
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
use bsnext_input::target::TargetKind;
use bsnext_input::InputWriter;
use bsnext_input::{Input, InputError};
use std::path::{Path, PathBuf};

pub mod start_from_example;
pub mod start_from_inputs;
pub mod start_from_paths;

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum StartKind {
    FromInput(StartFromInput),
    FromInputPaths(StartFromInputPaths),
    FromExample(StartFromExample),
    FromTrailingArgs(StartFromTrailingArgs),
}

impl StartKind {
    pub fn from_args(fs_opts: &FsOpts, input_opts: &InputOpts, cmd: &StartCommand) -> Self {
        // todo: re-implement example command
        // if let Some(example) = args.example {
        //     return StartKind::FromExample(StartFromExample {
        //         example,
        //         write_input: args.write,
        //         port: args.port,
        //         temp: args.temp,
        //         name: args.name.clone(),
        //         target_kind: args
        //             .target
        //             .as_ref()
        //             .map(ToOwned::to_owned)
        //             .unwrap_or_default(),
        //         dir: args.dir.clone(),
        //         force: args.force,
        //     });
        // }

        // todo: make the addition of a proxy + route opts easier?
        if cmd.trailing.is_empty() {
            tracing::debug!("0 trailing, {} inputs", input_opts.input.len());
            if input_opts.input.is_empty() && !cmd.proxies.is_empty() {
                let first_proxy = cmd.proxies.first().expect("guarded first proxy");
                let r = Route::proxy(first_proxy);
                let id = ServerIdentity::from_port_or_named(cmd.port).unwrap_or_else(|_e| {
                    tracing::error!("A problem occurred with the port?");
                    ServerIdentity::named()
                });
                let ser = ServerConfig::from_route(r, id);
                let input = Input::from_server(ser);
                StartKind::FromInput(StartFromInput { input })
            } else {
                StartKind::FromInputPaths(StartFromInputPaths {
                    input_paths: input_opts.input.clone(),
                    port: cmd.port,
                })
            }
        } else {
            StartKind::FromTrailingArgs(StartFromTrailingArgs {
                paths: cmd.trailing.clone(),
                write_input: fs_opts.write,
                port: cmd.port,
                force: fs_opts.force,
                route_opts: Opts {
                    cors: cmd.cors.then_some(CorsOpts::Cors(true)),
                    ..Default::default()
                },
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
            StartKind::FromTrailingArgs(from_trailing_args) => from_trailing_args.input(ctx),
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
