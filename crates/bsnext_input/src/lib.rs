use crate::target::TargetKind;

use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::read_to_string;
use std::net::AddrParseError;
use std::path::{Path, PathBuf};

use crate::md::MarkdownError;
use crate::yml::YamlError;

#[cfg(test)]
pub mod input_test;
pub mod md;
pub mod paths;
pub mod route;
pub mod route_manifest;
pub mod server_config;
pub mod target;
#[cfg(test)]
pub mod watch_opt_test;
pub mod yml;

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct Input {
    pub servers: Vec<server_config::ServerConfig>,
}

impl Input {
    pub fn from_input_path<P: AsRef<Path>>(path: P) -> Result<Self, InputError> {
        match path.as_ref().extension().and_then(|x| x.to_str()) {
            None => Err(InputError::MissingExtension(path.as_ref().to_owned())),
            Some("yml") | Some("yaml") => Input::from_yaml_path(path),
            Some("md") | Some("markdown") => Input::from_md_path(path),
            Some(other) => Err(InputError::UnsupportedExtension(other.to_string())),
        }
    }
    fn from_yaml_path<P: AsRef<Path>>(path: P) -> Result<Self, InputError> {
        let str = read_to_string(&path)?;
        let output = serde_yaml::from_str::<Self>(str.as_str()).map_err(move |e| {
            if let Some(location) = e.location() {
                YamlError::ParseErrorWithLocation {
                    serde_error: e,
                    input: str,
                    path: path.as_ref().to_string_lossy().to_string(),
                    line: location.line(),
                    column: location.column(),
                }
            } else {
                YamlError::ParseError {
                    serde_error: e,
                    input: str,
                    path: path.as_ref().to_string_lossy().to_string(),
                }
            }
        })?;
        // todo: don't allow duplicates?.
        Ok(output)
    }
    fn from_md_path<P: AsRef<Path>>(path: P) -> Result<Self, InputError> {
        let str = read_to_string(path)?;
        let input = md::md_to_input(&str)?;
        // todo: don't allow duplicates.
        Ok(input)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InputError {
    #[error("no suitable inputs could be found")]
    MissingInputs,
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
    #[error("InputWriteError prevented startup {0}")]
    InputWriteError(#[from] InputWriteError),
    #[error("Input path error prevented startup {0}")]
    PathError(#[from] PathError),
    #[error("Input port error prevented startup {0}")]
    PortError(#[from] PortError),
    #[error("Input directory error prevented startup {0}")]
    DirError(#[from] DirError),
    #[error("Markdown error: {0}")]
    MarkdownError(#[from] MarkdownError),
    #[error("{0}")]
    YamlError(#[from] YamlError),
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

#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
pub struct PathDefs(Vec<PathDefinition>);

#[derive(Debug, thiserror::Error, serde::Serialize, serde::Deserialize)]
struct PathDefinition {
    pub input: String,
    pub cwd: PathBuf,
    pub absolute: PathBuf,
}

impl Display for PathDefs {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for pd in self.0.iter() {
            writeln!(f, "  cwd:   {}", pd.cwd.display())?;
            writeln!(f, "  input: {}", pd.input)?;
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
        TargetKind::Yaml => "input.yml",
        TargetKind::Toml => "input.toml",
        TargetKind::Md => "input.md",
    };
    let next_path = cwd.join(name);
    fs::write(&next_path, string)
        .map(|()| next_path.clone())
        .map_err(|_e| InputWriteError::FailedWrite { path: next_path })
}
