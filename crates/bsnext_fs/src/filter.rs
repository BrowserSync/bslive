use crate::PathDescription;
use glob_match::glob_match;

#[derive(Debug, Clone)]
pub enum Filter {
    None,
    Extension { ext: String },
    Glob { glob: String },
}

impl PathFilter for Filter {
    fn filter(&self, pd: &PathDescription) -> bool {
        match self {
            Filter::None => false,
            Filter::Extension { ext } => pd
                .absolute
                .extension()
                .is_some_and(|x| x.to_string_lossy() == *ext),
            Filter::Glob { glob } => {
                let target = pd.relative.unwrap_or(pd.absolute);
                let did_match = glob_match(glob, target.to_string_lossy().to_string().as_str());
                did_match
            }
        }
    }
}

pub trait PathFilter {
    fn filter(&self, pd: &PathDescription) -> bool;
}
