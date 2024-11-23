use crate::server_config::ServerIdentity;
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

#[derive(Debug)]
pub struct InputSourceFile {
    path: PathBuf,
    content: String,
}

impl InputSourceFile {
    pub fn new(path: impl Into<PathBuf>, content: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            content: content.into(),
        }
    }

    pub fn path(&self) -> &'_ Path {
        &self.path
    }
    pub fn content(&self) -> &str {
        &self.content
    }
}

#[derive(Debug)]
pub enum InputSourceKind {
    Type(Input),
    File {
        src_file: InputSourceFile,
        input: Input,
    },
}

pub trait InputSource {
    fn into_input(self, _identity: Option<ServerIdentity>) -> InputSourceKind
    where
        Self: Sized,
    {
        InputSourceKind::Type(Default::default())
    }
}

#[derive(Debug, Default, Clone)]
pub struct InputCtx {
    prev_server_ids: Option<Vec<ServerIdentity>>,
}

impl InputCtx {
    pub fn new(servers: &[ServerIdentity]) -> Self {
        if servers.is_empty() {
            Self::default()
        } else {
            Self {
                prev_server_ids: Some(servers.to_vec()),
            }
        }
    }

    pub fn server_ids(&self) -> Option<&[ServerIdentity]> {
        self.prev_server_ids.as_ref().map(Vec::as_slice)
    }

    pub fn first_id_or_default(&self) -> ServerIdentity {
        self.prev_server_ids
            .as_ref()
            .and_then(|x| x.get(0))
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| ServerIdentity::named())
    }
}

pub trait InputCreation {
    fn from_input_path<P: AsRef<Path>>(path: P, ctx: &InputCtx) -> Result<Input, Box<InputError>>;
    fn from_input_str<P: AsRef<str>>(content: P, ctx: &InputCtx) -> Result<Input, Box<InputError>>;
}

pub trait InputWriter {
    fn input_to_str(&self, input: &Input) -> String;
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
    #[error("HTML error: {0}")]
    HtmlError(String),
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
    #[error("couldn't read the status of {path}")]
    CannotQueryStatus { path: PathBuf },
    #[error("input already exists, override with --force (dangerous) {path}")]
    Exists { path: PathBuf },
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
    #[error("could not query the status")]
    CannotQueryStatus { path: PathBuf },
    #[error("directory already exists, override with --force (dangerous) {path}")]
    Exists { path: PathBuf },
}

impl From<DirError> for Box<InputError> {
    fn from(value: DirError) -> Self {
        Box::new(InputError::DirError(value))
    }
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
