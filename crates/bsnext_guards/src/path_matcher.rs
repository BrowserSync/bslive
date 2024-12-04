use http::Uri;
use std::str::FromStr;
use urlpattern::UrlPatternInit;
use urlpattern::UrlPatternMatchInput;
use urlpattern::{UrlPattern, UrlPatternOptions};

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum PathMatcher {
    Str(String),
    Def(PathMatcherDef),
}

impl PathMatcher {
    pub fn str(str: impl Into<String>) -> Self {
        Self::Str(str.into())
    }
    pub fn pathname(str: impl Into<String>) -> Self {
        Self::Def(PathMatcherDef {
            pathname: Some(str.into()),
            search: None,
        })
    }
    pub fn query(str: impl Into<String>) -> Self {
        Self::Def(PathMatcherDef {
            pathname: None,
            search: Some(str.into()),
        })
    }
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct PathMatcherDef {
    pub(crate) pathname: Option<String>,
    pub(crate) search: Option<String>,
}

impl PathMatcher {
    pub fn test_uri(&self, uri: &Uri) -> bool {
        let Some(path_and_query) = uri.path_and_query() else {
            tracing::error!("how is this possible?");
            return false;
        };

        let path = path_and_query.path();
        let seary = path_and_query.query();

        let incoming = UrlPatternInit {
            pathname: Some(path.into()),
            search: seary.map(ToOwned::to_owned),
            ..Default::default()
        };

        // convert the config into UrlPatternInit
        // example: /style.css
        let matching_options: UrlPatternInit = match self {
            PathMatcher::Str(str) => {
                if let Ok(uri) = &Uri::from_str(str) {
                    if let Some(pq) = uri.path_and_query() {
                        let path = pq.path();
                        let query = pq.query();
                        UrlPatternInit {
                            pathname: Some(path.into()),
                            search: query.map(ToOwned::to_owned),
                            ..Default::default()
                        }
                    } else {
                        tracing::error!("could not parse the matching string you gave {}", str);
                        Default::default()
                    }
                } else {
                    tracing::error!("could not parse the matching string you gave {}", str);
                    Default::default()
                }
            }
            PathMatcher::Def(PathMatcherDef { pathname, search }) => UrlPatternInit {
                pathname: pathname.to_owned(),
                search: search.to_owned(),
                ..Default::default()
            },
        };
        let opts = UrlPatternOptions::default();
        let Ok(pattern) = <UrlPattern>::parse(matching_options, opts) else {
            tracing::error!("could not parse the input");
            return false;
        };
        // dbg!(&incoming);
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
    use crate::path_matcher::PathMatcher;
    use http::Uri;

    #[test]
    fn test_url_pattern_pathname() {
        let pm = PathMatcher::pathname("/");
        assert_eq!(pm.test_uri(&Uri::from_static("/")), true);
        let pm = PathMatcher::pathname("/*.css");
        assert_eq!(pm.test_uri(&Uri::from_static("/style.css")), true);
        let pm = PathMatcher::pathname("/here/*.css");
        assert_eq!(pm.test_uri(&Uri::from_static("/style.css")), false);
        let pm = PathMatcher::pathname("/**/*.css");
        assert_eq!(pm.test_uri(&Uri::from_static("/style.css")), true);
        let pm = PathMatcher::pathname("/**/*.css");
        assert_eq!(
            pm.test_uri(&Uri::from_static("/a/b/c/--oopasxstyle.css")),
            true
        );
        assert_eq!(
            pm.test_uri(&Uri::from_static("/a/b/c/--oopasxstyle.html")),
            false
        );
    }
    #[test]
    fn test_url_pattern_query() {
        let pm = PathMatcher::str("/?abc=true");
        assert_eq!(pm.test_uri(&Uri::from_static("/")), false);
        assert_eq!(pm.test_uri(&Uri::from_static("/?def=true")), false);
        assert_eq!(pm.test_uri(&Uri::from_static("/?abc=true")), true);
        assert_eq!(pm.test_uri(&Uri::from_static("/?abc=")), false);

        let pm2 = PathMatcher::str("/**/*?delayms");
        assert_eq!(pm2.test_uri(&Uri::from_static("/?delayms")), true);

        let pm2 = PathMatcher::query("?*a*b*c*foo=bar");
        assert_eq!(
            pm2.test_uri(&Uri::from_static("/?delay.ms=2000&a-b-c-foo=bar")),
            true
        );
    }
    #[test]
    fn test_url_pattern_str() {
        let pm = PathMatcher::str("/");
        assert_eq!(pm.test_uri(&Uri::from_static("/")), true);
        let pm = PathMatcher::str("/*.css");
        assert_eq!(pm.test_uri(&Uri::from_static("/style.css")), true);
        let pm = PathMatcher::str("/here/*.css");
        assert_eq!(pm.test_uri(&Uri::from_static("/style.css")), false);
        let pm = PathMatcher::str("/**/*.css");
        assert_eq!(pm.test_uri(&Uri::from_static("/style.css")), true);
        let pm = PathMatcher::str("/**/*.css");
        assert_eq!(
            pm.test_uri(&Uri::from_static("/a/b/c/--oopasxstyle.css")),
            true
        );
        assert_eq!(
            pm.test_uri(&Uri::from_static("/a/b/c/--oopasxstyle.html")),
            false
        );
    }
}
