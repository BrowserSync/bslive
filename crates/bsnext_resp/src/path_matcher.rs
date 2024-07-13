use urlpattern::UrlPattern;
use urlpattern::UrlPatternInit;
use urlpattern::UrlPatternMatchInput;

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum PathMatcher {
    Str(String),
    Def(PathMatcherDef),
}

impl PathMatcher {
    pub fn pathname(str: impl Into<String>) -> Self {
        Self::Def(PathMatcherDef {
            pathname: Some(str.into()),
        })
    }
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct PathMatcherDef {
    pub pathname: Option<String>,
}

impl PathMatcher {
    pub fn test(&self, uri: &str) -> bool {
        let incoming = UrlPatternInit {
            pathname: Some(uri.to_owned()),
            ..Default::default()
        };

        let to_pathname = match self {
            PathMatcher::Str(str) => str.as_str(),
            PathMatcher::Def(PathMatcherDef {
                pathname: Some(str),
            }) => str.as_str(),
            PathMatcher::Def(PathMatcherDef { pathname: None }) => {
                unreachable!("how can this occur?")
            }
        };
        tracing::trace!(?to_pathname, ?uri, "PathMatcher::Str");
        let init = UrlPatternInit {
            pathname: Some(to_pathname.to_owned()),
            ..Default::default()
        };
        let Ok(pattern) = <UrlPattern>::parse(init) else {
            tracing::error!(?to_pathname, "could not parse the input");
            return false;
        };
        match pattern.test(UrlPatternMatchInput::Init(incoming)) {
            Ok(true) => {
                tracing::trace!("matched!");
                true
            }
            Ok(false) => {
                tracing::trace!("not matched!");
                false
            }
            Err(e) => {
                tracing::error!("could not match {:?}", e);
                false
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::inject_addition::{AdditionPosition, InjectAddition};
    use crate::inject_opts::{InjectOpts, Injection, InjectionItem, MatcherList};

    #[test]
    fn test_path_matchers() {
        #[derive(Debug, serde::Deserialize)]
        struct A {
            inject: InjectOpts,
        }
        let input = r#"
inject:
    append: lol
    only:
      - /*.css
      - pathname: /*.css
"#;
        let expected = A {
            inject: InjectOpts::Item(InjectionItem {
                inner: Injection::Addition(InjectAddition {
                    addition_position: AdditionPosition::Append("lol".to_string()),
                }),
                only: Some(MatcherList::Items(vec![
                    PathMatcher::Str("/*.css".to_string()),
                    PathMatcher::Def(PathMatcherDef {
                        pathname: Some("/*.css".to_string()),
                    }),
                ])),
            }),
        };
        let actual: Result<A, _> = serde_yaml::from_str(input);
        assert_eq!(actual.unwrap().inject, expected.inject);
    }

    #[test]
    fn test_path_matcher_single() {
        #[derive(Debug, serde::Deserialize)]
        struct A {
            inject: InjectOpts,
        }
        let input = r#"
    inject:
        append: lol
        only: /*.css
    "#;
        let expected = A {
            inject: InjectOpts::Item(InjectionItem {
                inner: Injection::Addition(InjectAddition {
                    addition_position: AdditionPosition::Append("lol".to_string()),
                }),
                only: Some(MatcherList::Item(PathMatcher::Str("/*.css".to_string()))),
            }),
        };
        let actual: Result<A, _> = serde_yaml::from_str(input);
        assert_eq!(actual.unwrap().inject, expected.inject);
    }

    #[test]
    fn test_url_pattern() {
        let pm = PathMatcher::pathname("/");
        assert_eq!(pm.test("/"), true);
        let pm = PathMatcher::pathname("/*.css");
        assert_eq!(pm.test("/style.css"), true);
        let pm = PathMatcher::pathname("/here/*.css");
        assert_eq!(pm.test("/style.css"), false);
        let pm = PathMatcher::pathname("/**/*.css");
        assert_eq!(pm.test("/style.css"), true);
        let pm = PathMatcher::pathname("/**/*.css");
        assert_eq!(pm.test("/a/b/c/--oopasxstyle.css"), true);
        assert_eq!(pm.test("/a/b/c/--oopasxstyle.html"), false);
    }
}
