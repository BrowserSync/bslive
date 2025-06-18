use crate::PathDescription;
use glob_match::glob_match;

#[derive(Debug, Clone)]
pub enum Filter {
    None,
    Extension { ext: String },
    Glob { glob: String },
    Any { any: String },
}

impl PathFilter for Filter {
    fn filter(&self, pd: &PathDescription) -> bool {
        match self {
            Filter::None => false,
            Filter::Extension { ext } => {
                tracing::trace!("testing extension `{:?}` against `{}`", pd, ext);
                pd.absolute
                    .extension()
                    .is_some_and(|x| x.to_string_lossy() == *ext)
            }
            Filter::Glob { glob } => {
                let target = pd.relative.unwrap_or(pd.absolute);
                let compare = target.to_string_lossy().to_string();
                let did_match = glob_match(glob, &compare);
                tracing::trace!(
                    "testing glob `{}` against `{}`: {}",
                    glob,
                    compare.as_str(),
                    did_match
                );
                did_match
            }
            Filter::Any { any } => {
                let did_match = pd.absolute.to_string_lossy().contains(any);
                tracing::trace!(
                    "testing Filter::Any `{:?}` against `{}` = {did_match}",
                    any,
                    pd.absolute.display(),
                );
                did_match
            }
        }
    }
}

pub trait PathFilter {
    fn filter(&self, pd: &PathDescription) -> bool;
}
