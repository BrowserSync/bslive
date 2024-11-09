use crate::target::TargetKind;

use crate::md::MarkdownError;
use crate::yml::YamlError;
use miette::{JSONReportHandler, NamedSource};
use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::read_to_string;
use std::net::AddrParseError;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub mod client_config;
#[cfg(test)]
pub mod input_test;
pub mod md;
pub mod path_def;
pub mod paths;
pub mod playground;
pub mod route;
pub mod route_manifest;
pub mod server_config;
pub mod startup;
pub mod target;
#[cfg(test)]
pub mod watch_opt_test;
pub mod watch_opts;
pub mod yml;

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct Input {
    pub servers: Vec<server_config::ServerConfig>,
}

impl FromStr for Input {
    type Err = InputError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_yaml::from_str::<Self>(s).map_err(move |e| {
            let err = if let Some(location) = e.location() {
                YamlError::ParseRawInputErrorWithLocation {
                    serde_error: e,
                    input: s.to_string(),
                    line: location.line(),
                    column: location.column(),
                }
            } else {
                YamlError::ParseRawInputError {
                    serde_error: e,
                    input: s.to_string(),
                }
            };
            InputError::YamlError(err)
        })
    }
}

impl Input {
    pub fn from_input_path<P: AsRef<Path>>(path: P) -> Result<Self, Box<InputError>> {
        match path.as_ref().extension().and_then(|x| x.to_str()) {
            None => Err(Box::new(InputError::MissingExtension(
                path.as_ref().to_owned(),
            ))),
            Some("yml") | Some("yaml") => Input::from_yaml_path(path),
            Some("md") | Some("markdown") => Input::from_md_path(path),
            Some(other) => Err(Box::new(InputError::UnsupportedExtension(
                other.to_string(),
            ))),
        }
    }
    fn from_yaml_path<P: AsRef<Path>>(path: P) -> Result<Self, Box<InputError>> {
        let str = read_to_string(&path).map_err(|e| Box::new(e.into()))?;
        if str.trim().is_empty() {
            return Err(Box::new(InputError::YamlError(YamlError::EmptyError {
                path: path.as_ref().to_string_lossy().to_string(),
            })));
        }
        let output = serde_yaml::from_str::<Self>(str.as_str())
            .map_err(move |e| {
                if let Some(loc) = e.location() {
                    BsLiveRulesError {
                        err_span: (loc.index()..loc.index() + 1).into(),
                        src: NamedSource::new(path.as_ref().to_string_lossy().to_string(), str),
                        message: e.to_string(),
                        summary: None,
                    }
                } else {
                    unreachable!("handle later")
                }
            })
            .map_err(|e| Box::new(e.into()))?;
        // todo: don't allow duplicates?.
        Ok(output)
    }
    fn from_md_path<P: AsRef<Path>>(path: P) -> Result<Self, Box<InputError>> {
        let str = read_to_string(path).map_err(|e| Box::new(e.into()))?;
        let input = md::md_to_input(&str).map_err(|e| Box::new(e.into()))?;
        // todo: don't allow duplicates.
        Ok(input)
    }
}

#[derive(Debug, miette::Diagnostic, thiserror::Error)]
pub enum InputError {
    #[error("no suitable inputs could be found")]
    MissingInputs,
    #[error("input file is empty")]
    EmptyInput,
    #[error("could not read input, error: {0}")]
    InvalidInput(String),
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("Could not find the input file: {0}")]
    NotFound(PathBuf),
    #[error("Paths without extensions are not supported: {0}")]
    MissingExtension(PathBuf),
    #[error("Unsupported extension: {0}")]
    UnsupportedExtension(String),
    #[error("{0}")]
    InputWriteError(#[from] InputWriteError),
    #[error("{0}")]
    PathError(#[from] PathError),
    #[error("{0}")]
    PortError(#[from] PortError),
    #[error("{0}")]
    DirError(#[from] DirError),
    #[error("Markdown error: {0}")]
    MarkdownError(#[from] MarkdownError),
    #[error("{0}")]
    YamlError(#[from] YamlError),
    #[error(transparent)]
    #[diagnostic(transparent)]
    BsLiveRules(#[from] BsLiveRulesError),
}

#[derive(Debug, thiserror::Error)]
pub enum WatchError {
    #[error("don't add `.` before the extension.")]
    InvalidExtensionFilter,
    #[error("empty")]
    EmptyExtensionFilter,
}

#[derive(Debug, thiserror::Error)]
pub enum InputWriteError {
    #[error("couldn't write input to {path}")]
    FailedWrite { path: PathBuf },
}

#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub enum PathError {
    #[error("path(s) not found \n{paths}")]
    MissingPaths { paths: PathDefs },
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum PortError {
    #[error("could not use that port: {port} {err}")]
    InvalidPort { port: u16, err: AddrParseError },
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum DirError {
    #[error("could not create that dir: {path}")]
    CannotCreate { path: PathBuf },
    #[error("could not change the process CWD to: {path}")]
    CannotMove { path: PathBuf },
}

#[derive(miette::Diagnostic, Debug, thiserror::Error)]
#[error("bslive rules violated")]
#[diagnostic()]
pub struct BsLiveRulesError {
    // Note: label but no source code
    #[label = "{message}"]
    err_span: miette::SourceSpan,
    #[source_code]
    src: miette::NamedSource<String>,
    message: String,
    #[help]
    summary: Option<String>,
}

impl BsLiveRulesError {
    pub fn as_string(&self) -> String {
        let n = miette::GraphicalReportHandler::new();
        let mut inner = String::new();
        n.render_report(&mut inner, self).expect("write?");
        inner
    }

    pub fn as_json(&self) -> String {
        let mut out = String::new();
        JSONReportHandler::new()
            .render_report(&mut out, self)
            .unwrap();
        out
    }
}

#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub struct PathDefs(pub Vec<PathDefinition>);

#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub struct PathDefinition {
    pub input: String,
    pub cwd: PathBuf,
    pub absolute: PathBuf,
}

impl Display for PathDefs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for pd in self.0.iter() {
            writeln!(f, "cwd:   {}", pd.cwd.display())?;
            writeln!(f, "input: {}", pd.input)?;
        }
        Ok(())
    }
}
impl Display for PathDefinition {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PathDefinition").finish()
    }
}

pub fn rand_word() -> String {
    random_word::gen(random_word::Lang::En).to_string()
}

pub fn fs_write_input(
    cwd: &Path,
    input: &Input,
    target_kind: TargetKind,
) -> Result<PathBuf, InputWriteError> {
    let string = match target_kind {
        TargetKind::Yaml => serde_yaml::to_string(&input).expect("create yaml?"),
        TargetKind::Toml => toml::to_string_pretty(&input).expect("create toml?"),
        TargetKind::Md => md::input_to_str(input),
    };
    let name = match target_kind {
        TargetKind::Yaml => "bslive.yml",
        TargetKind::Toml => "bslive.toml",
        TargetKind::Md => "bslive.md",
    };
    let next_path = cwd.join(name);
    fs::write(&next_path, string)
        .map(|()| next_path.clone())
        .map_err(|_e| InputWriteError::FailedWrite { path: next_path })
}
