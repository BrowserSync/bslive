use crate::path_matcher::PathMatcher;

pub mod path_matcher;
pub mod route_guard;

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum MatcherList {
    None,
    Item(PathMatcher),
    Items(Vec<PathMatcher>),
}
