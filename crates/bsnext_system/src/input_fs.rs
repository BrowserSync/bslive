use bsnext_input::{Input, InputCreation, InputCtx, InputError};
use std::path::{Path, PathBuf};

pub enum ResolvedInputOutcome {
    /// You provided a path that couldn't be resolved
    Missing {
        user_input: String,
        cwd: PathBuf,
        absolute: PathBuf,
        err: Box<InputError>,
    },

    GivenPath {
        user_input: String,
        cwd: PathBuf,
        absolute: PathBuf,
    },

    /// You provided no paths but a path was auto-resolved
    Auto {
        named: String,
        cwd: PathBuf,
        absolute: PathBuf,
    },

    /// You provided no paths and nothing was auto-resolved
    Empty,
}

impl ResolvedInputOutcome {
    pub fn new<T: AsRef<str>>(cwd: PathBuf, inputs: &[T]) -> Self {
        resolve_inputs(cwd, inputs)
    }
}

fn resolve_inputs<T: AsRef<str>>(cwd: PathBuf, inputs: &[T]) -> ResolvedInputOutcome {
    let input_candidates = inputs
        .iter()
        .map(|path| {
            let abs = cwd.join(path.as_ref());
            let exists = Path::exists(&abs);
            (path.as_ref(), abs, exists)
        })
        .collect::<Vec<(&str, PathBuf, bool)>>();

    if let Some((user_input, abs, exists)) = input_candidates.first() {
        if !exists {
            return ResolvedInputOutcome::Missing {
                user_input: (*user_input).to_owned(),
                absolute: (*abs).to_owned(),
                cwd,
                err: Box::new(InputError::MissingInputs),
            };
        }
        return ResolvedInputOutcome::GivenPath {
            user_input: (*user_input).to_owned(),
            absolute: (*abs).to_owned(),
            cwd,
        };
    }

    let auto_lookups = ["bslive.yml", "bslive.yaml", "bslive.md", "bslive.html"];

    for named in auto_lookups {
        let abs = cwd.join(named);
        let exists = Path::exists(&abs);
        if !exists {
            continue;
        }
        return ResolvedInputOutcome::Auto {
            named: (*named).to_owned(),
            cwd,
            absolute: (*abs).to_owned(),
        };
    }

    ResolvedInputOutcome::Empty
}

pub fn from_input_path<P: AsRef<Path>>(path: P, ctx: &InputCtx) -> Result<Input, Box<InputError>> {
    match path.as_ref().extension().and_then(|x| x.to_str()) {
        None => Err(Box::new(InputError::MissingExtension(
            path.as_ref().to_owned(),
        ))),
        Some("yml") | Some("yaml") => bsnext_yaml::yaml_fs::YamlFs::from_input_path(path, ctx),
        Some("md") | Some("markdown") => bsnext_md::md_fs::MdFs::from_input_path(path, ctx),
        Some("html") => bsnext_html::HtmlFs::from_input_path(path, ctx),
        Some("js") => bsnext_js::JsFs::from_input_path(path, ctx),
        Some(other) => Err(Box::new(InputError::UnsupportedExtension(
            other.to_string(),
        ))),
    }
}
