use crate::run::RunCommand;
use crate::start::start_command::StartCommand;
use crate::start::start_kind::run_from_input::RunFromInputPaths;
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
    Run(RunFromInputPaths),
    FromInput(StartFromInput),
    FromInputPaths(StartFromInputPaths),
    FromExample(StartFromExample),
    FromTrailingArgs(StartFromTrailingArgs),
}

impl StartKind {
    #[tracing::instrument(name = "from_args")]
    pub fn from_args(fs_opts: &FsOpts, input_opts: &InputOpts, cmd: &StartCommand) -> Self {
        // todo: make the addition of a proxy + route opts easier?
        if !cmd.trailing.is_empty() {
            tracing::debug!(
                "{} trailing, {} inputs",
                cmd.trailing.len(),
                input_opts.input.len()
            );
            return StartKind::FromTrailingArgs(StartFromTrailingArgs {
                paths: cmd.trailing.clone(),
                write_input: fs_opts.write,
                port: cmd.port,
                force: fs_opts.force,
                watch_sub_opts: cmd.watch_sub_opts.clone(),
                route_opts: Opts {
                    cors: cmd.cors.then_some(CorsOpts::Cors(true)),
                    ..Default::default()
                },
            });
        }

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
    }
    pub fn from_input(input: Input) -> Self {
        Self::FromInput(StartFromInput { input })
    }

    #[tracing::instrument(skip_all)]
    pub fn from_run_args(_fs_opts: &FsOpts, input_opts: &InputOpts, run_cmd: RunCommand) -> Self {
        let from_cmd = run_cmd.as_input();

        tracing::debug!(run_cmd.trailing = ?run_cmd.trailing);
        tracing::debug!(run_cmd.sh_commands = ?run_cmd.sh_commands);
        tracing::debug!(run_cmd.all = ?run_cmd.all);

        let named = if run_cmd.trailing.is_empty() {
            vec!["default".to_string()]
        } else {
            run_cmd.trailing
        };

        // dry takes precedence
        let run_mode = if run_cmd.dry {
            RunMode::Dry
        } else {
            RunMode::Exec {
                preview: run_cmd.preview,
                summary: run_cmd.summary,
            }
        };
        let top_level = TopLevelRunMode::Seq;
        StartKind::Run(RunFromInputPaths::new(
            from_cmd,
            input_opts.input.clone(),
            named,
            run_mode,
            top_level,
        ))
    }
}

impl SystemStart for StartKind {
    fn resolve_input(&self, ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        match self {
            StartKind::FromInputPaths(from_inputs) => from_inputs.resolve_input(ctx),
            StartKind::FromExample(from_example) => from_example.resolve_input(ctx),
            StartKind::FromTrailingArgs(from_trailing_args) => {
                from_trailing_args.resolve_input(ctx)
            }
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
