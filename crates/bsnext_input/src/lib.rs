use crate::yml::YamlError;
use miette::JSONReportHandler;
use std::fmt::{Display, Formatter};
use std::net::AddrParseError;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub mod client_config;
#[cfg(test)]
pub mod input_test;
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

pub trait InputCreation {
    fn from_input_path<P: AsRef<Path>>(path: P) -> Result<Input, Box<InputError>>;
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
    MarkdownError(String),
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
    pub err_span: miette::SourceSpan,
    #[source_code]
    pub src: miette::NamedSource<String>,
    pub message: String,
    #[help]
    pub summary: Option<String>,
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
