use crate::PathDescription;

#[derive(Debug, Clone)]
pub enum Filter {
    None,
    Extension {
        ext: String,
    },
    Glob {
        glob: globset::GlobMatcher,
        raw: String,
        scope: FilterScope,
    },
    Any {
        any: String,
    },
}
#[derive(Debug, Clone)]
pub enum FilterScope {
    Abs,
    Rel,
}

impl PathFilter for Filter {
    fn any(&self, pd: &PathDescription) -> bool {
        match self {
            Filter::None => false,
            Filter::Extension { ext } => {
                tracing::trace!("testing extension `{:?}` against `{}`", pd, ext);
                pd.absolute
                    .extension()
                    .is_some_and(|x| x.to_string_lossy() == *ext)
            }
            Filter::Glob { glob, scope, raw } => {
                let target = match (scope, pd.relative) {
                    (FilterScope::Rel, Some(rel)) => rel,
                    _ => pd.absolute,
                };
                let compare = target.to_string_lossy().to_string();

                let did_match = glob.is_match(&compare);
                tracing::debug!(
                    "testing glob `{}` against `{}`: {}",
                    raw,
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
    fn any(&self, pd: &PathDescription) -> bool;
}
