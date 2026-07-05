use crate::path_monitor;
use bsnext_fs::PathDescription;
use bsnext_fs::filter::PathFilter;
use bsnext_input::route::PathPattern;
use std::path::Path;

pub(crate) struct PathAndFilter<'a> {
    pub(crate) path: &'a Path,
    pub(crate) filter_kind: Option<PathPattern>,
}

impl<'a> PathAndFilter<'a> {
    pub fn new(p: &'a str) -> Self {
        match p.split_once("*") {
            // for cases like '**/*.toml' or '*.css'
            Some(("", ..)) => PathAndFilter {
                path: Path::new("."),
                filter_kind: Some(PathPattern::Glob {
                    glob: p.to_string(),
                }),
            },
            Some((before, ..)) => PathAndFilter {
                path: Path::new(before),
                filter_kind: Some(PathPattern::Glob {
                    glob: p.to_string(),
                }),
            },
            None => PathAndFilter {
                path: Path::new(p),
                filter_kind: None,
            },
        }
    }
}

impl PathFilter for PathAndFilter<'_> {
    fn any(&self, pd: &PathDescription) -> bool {
        match &self.filter_kind {
            None => {
                if self.path == pd.absolute {
                    return true;
                }
                pd.relative.map(|rel| rel == self.path).unwrap_or(false)
            }
            Some(filter) => {
                let filters = path_monitor::pattern_to_filter_list(filter);
                filters.iter().any(|x| x.any(pd))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_path_and_filter_with_glob() {
        let input = "abc/*.css";
        let result = PathAndFilter::new(input);

        assert_eq!(result.path, Path::new("abc/"));
        assert!(matches!(result.filter_kind, Some(PathPattern::Glob { .. })));

        if let Some(PathPattern::Glob { glob }) = result.filter_kind {
            assert_eq!(glob, "abc/*.css");
        }
    }

    #[test]
    fn test_path_and_filter_without_glob() {
        let input = "abc/style.css";
        let result = PathAndFilter::new(input);

        assert_eq!(result.path, Path::new("abc/style.css"));
        assert!(result.filter_kind.is_none());
    }

    #[test]
    fn test_glob_without_path() {
        let input = "**/*.toml";
        let result = PathAndFilter::new(input);

        assert_eq!(result.path, Path::new("."));
        // assert!(result.filter_kind.is_none());
    }
    #[test]
    fn test_e2e_filtering() {
        use bsnext_fs::{Abs, Cwd};
        let abs = Abs("/user/shakyshane/abc/style.css");
        let cwd = Cwd("/user/shakyshane");
        let dirs = [
            (&abs, &cwd, "abc/*.css", true),
            (&abs, &cwd, "**/*.css", true),
            (&abs, &cwd, "def/*.css", false),
            (&abs, &cwd, "abc/style.css", true),
            (&abs, &cwd, "/user/shakyshane/abc/style.css", true),
            (&abs, &cwd, "/user/shakyshane/abc/*.css", true),
            (&abs, &cwd, "/user/shakyshane/abc/*.{css,txt}", true),
            (&abs, &cwd, "/user/shakyshane/abc/*.{txt}", false),
            (&abs, &cwd, "/user/shakyshane/**/*.css", true),
            (&abs, &cwd, "/user/shakyshane/*.css", false),
            (&abs, &cwd, "**/abc/*.css", true),
            (&abs, &cwd, "**/def/*.css", false),
            (&abs, &cwd, "abc/**/*.css", true),
            (&abs, &cwd, "def/**/*.css", false),
            (&abs, &cwd, "*.css", false),
            (&abs, &cwd, "style.css", false),
            (&abs, &cwd, "abc/s*.css", true),
            (&abs, &cwd, "abc/style.*", true),
            (&abs, &cwd, "*/style.css", true),
        ];
        for (abs, cwd, dir, expected) in dirs {
            let change = PathDescription::from_cwd(abs, cwd);
            let v = PathAndFilter::new(dir);
            let actual = v.any(&change);
            assert_eq!(
                actual, expected,
                "dir was: {}, result should be {}",
                dir, expected
            );
        }
    }
}
