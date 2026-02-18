use crate::run::RunCommand;
use crate::start::start_command::StartCommand;
use crate::start::start_kind::run_from_input::RunFromInput;
use crate::start::start_kind::start_from_example::StartFromExample;
use crate::start::start_kind::start_from_inputs::{StartFromInput, StartFromInputPaths};
use crate::start::start_kind::start_from_paths::StartFromTrailingArgs;
use bsnext_core::shared_args::{FsOpts, InputOpts};
use bsnext_fs_helpers::{fs_write_str, FsWriteError, WriteMode};
use bsnext_input::route::{CorsOpts, Opts, Route};
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use bsnext_input::startup::{
    RunMode, StartupContext, SystemStart, SystemStartArgs, TopLevelRunMode,
};
use bsnext_input::target::TargetKind;
use bsnext_input::InputWriter;
use bsnext_input::{Input, InputError};
use std::path::{Path, PathBuf};

pub mod run_from_input;
pub mod start_from_example;
pub mod start_from_inputs;
pub mod start_from_paths;

#[allow(clippy::large_enum_variant)]
#[derive(Debug)]
pub enum StartKind {
    Run(RunFromInput),
    FromInput(StartFromInput),
    FromInputPaths(StartFromInputPaths),
    FromExample(StartFromExample),
    FromTrailingArgs(StartFromTrailingArgs),
}

impl StartKind {
    #[tracing::instrument(name = "from_args")]
    pub fn from_args(fs_opts: &FsOpts, input_opts: &InputOpts, cmd: &StartCommand) -> Self {
        // todo: make the addition of a proxy + route opts easier?
        if cmd.trailing.is_empty() {
            tracing::debug!("0 trailing, {} inputs", input_opts.input.len());
            if input_opts.input.is_empty() && !cmd.proxies.is_empty() {
                tracing::debug!("input was empty, but had proxies");
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
                tracing::debug!(
                    input_len = input_opts.input.len(),
                    proxes = cmd.proxies.len(),
                    "neither inputs nor proxies were present"
                );
                StartKind::FromInputPaths(StartFromInputPaths {
                    input_paths: input_opts.input.clone(),
                    port: cmd.port,
                })
            }
        } else {
            tracing::debug!(
                "{} trailing, {} inputs",
                cmd.trailing.len(),
                input_opts.input.len()
            );
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

    #[tracing::instrument(name = "from_run_args")]
    pub fn from_run_args(
        fs_opts: &FsOpts,
        input_opts: &InputOpts,
        run: RunCommand,
    ) -> Result<Self, Box<InputError>> {
        let maybe_input = StartFromInputPaths {
            input_paths: input_opts.input.clone(),
            port: None,
        };
        let input = maybe_input
            .input(&Default::default())
            .and_then(|def| match def {
                SystemStartArgs::PathWithInput { input, path } => Ok(Some(input)),
                SystemStartArgs::PathWithInvalidInput { path, input_error } => {
                    Err(Box::new(input_error))
                }
                SystemStartArgs::InputOnly { .. } => Ok(None),
                SystemStartArgs::RunOnly { .. } => Ok(None),
            })?;
        let from_cmd = run.as_input();
        let input = match input {
            None => from_cmd,
            Some(mut input_from_file) => {
                input_from_file.run.extend(from_cmd.run);
                input_from_file
            }
        };
        tracing::debug!(run.trailing = ?run.trailing);
        tracing::debug!(run.sh = ?run.sh_commands);
        let named = if run.trailing.is_empty() {
            vec!["default".to_string()]
        } else {
            run.trailing
        };
        let run_mode = if run.dry { RunMode::Dry } else { RunMode::Exec };
        let top_level = if run.all {
            TopLevelRunMode::All
        } else {
            TopLevelRunMode::Seq
        };
        let start_kind = StartKind::Run(RunFromInput::new(input, named, run_mode, top_level));
        Ok(start_kind)
    }
}

impl SystemStart for StartKind {
    fn input(&self, ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        match self {
            StartKind::FromInputPaths(from_inputs) => from_inputs.input(ctx),
            StartKind::FromExample(from_example) => from_example.input(ctx),
            StartKind::FromTrailingArgs(from_trailing_args) => from_trailing_args.input(ctx),
            StartKind::FromInput(from_inputs) => from_inputs.input(ctx),
            StartKind::Run(run_from_input) => run_from_input.input(ctx),
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
