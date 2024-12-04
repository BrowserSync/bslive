use crate::path_matcher::PathMatcher;
use http::Uri;

pub mod path_matcher;
pub mod route_guard;

#[derive(Debug, Default, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum MatcherList {
    #[default]
    None,
    Item(PathMatcher),
    Items(Vec<PathMatcher>),
}

impl MatcherList {
    pub fn test_uri(&self, uri: &Uri) -> bool {
        match self {
            MatcherList::None => true,
            MatcherList::Item(matcher) => matcher.test_uri(uri),
            MatcherList::Items(matchers) => matchers.iter().any(|m| m.test_uri(uri)),
        }
    }
}
