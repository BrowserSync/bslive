use crate::args::Args;
use crate::start_kind::start_from_example::StartFromExample;
use crate::start_kind::start_from_inputs::{StartFromInput, StartFromInputPaths};
use crate::start_kind::start_from_paths::StartFromDirPaths;
use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
use bsnext_input::{Input, InputError};

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
    pub fn from_args(args: Args) -> Self {
        if let Some(example) = args.example {
            return StartKind::FromExample(StartFromExample {
                example,
                write_input: args.write,
                port: args.port,
                temp: args.temp,
                name: args.name,
                target_kind: args.target.unwrap_or_default(),
                dir: args.dir.clone(),
                force: args.force,
            });
        }

        if !args.paths.is_empty() {
            StartKind::FromDirPaths(StartFromDirPaths {
                paths: args.paths,
                write_input: args.write,
                port: args.port,
                force: args.force,
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

pub mod start_fs {

    use std::fs;
    use std::path::{Path, PathBuf};

    use bsnext_input::target::TargetKind;
    use bsnext_input::{DirError, Input, InputWriteError, InputWriter};

    #[derive(Default, Debug, PartialEq)]
    pub enum WriteMode {
        #[default]
        Safe,
        Override,
    }
    pub fn fs_write_input(
        cwd: &Path,
        input: &Input,
        target_kind: TargetKind,
        write_mode: &WriteMode,
    ) -> Result<PathBuf, InputWriteError> {
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
        let next_path = cwd.join(name);
        tracing::info!(
            "✏️ writing {} bytes to {}",
            string.len(),
            next_path.display()
        );

        let exists = fs::exists(&next_path).map_err(|_e| InputWriteError::CannotQueryStatus {
            path: next_path.clone(),
        })?;

        if exists && *write_mode == WriteMode::Safe {
            return Err(InputWriteError::Exists { path: next_path });
        }

        fs::write(&next_path, string)
            .map(|()| next_path.clone())
            .map_err(|_e| InputWriteError::FailedWrite { path: next_path })
    }
    pub fn fs_write_input_src(
        cwd: &Path,
        path: &Path,
        string: &str,
        write_mode: &WriteMode,
    ) -> Result<PathBuf, InputWriteError> {
        let next_path = cwd.join(path);
        tracing::info!(
            "✏️ writing {} bytes to {}",
            string.len(),
            next_path.display()
        );

        let exists = fs::exists(&next_path).map_err(|_e| InputWriteError::CannotQueryStatus {
            path: next_path.clone(),
        })?;

        if exists && *write_mode == WriteMode::Safe {
            return Err(InputWriteError::Exists { path: next_path });
        }

        fs::write(&next_path, string)
            .map(|()| next_path.clone())
            .map_err(|_e| InputWriteError::FailedWrite { path: next_path })
    }

    pub fn create_dir(dir: &Path, write_mode: &WriteMode) -> Result<PathBuf, DirError> {
        let exists = fs::exists(dir).map_err(|_e| DirError::CannotQueryStatus {
            path: dir.to_path_buf(),
        })?;

        if exists && *write_mode == WriteMode::Safe {
            return Err(DirError::Exists {
                path: dir.to_path_buf(),
            });
        }

        fs::create_dir_all(dir)
            .map_err(|_e| DirError::CannotCreate {
                path: dir.to_path_buf(),
            })
            .and_then(|_pb| {
                std::env::set_current_dir(dir).map_err(|_e| DirError::CannotMove {
                    path: dir.to_path_buf(),
                })
            })
            .map(|_| dir.to_path_buf())
    }
}
