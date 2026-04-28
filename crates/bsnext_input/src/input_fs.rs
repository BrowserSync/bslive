use crate::InputError;
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
