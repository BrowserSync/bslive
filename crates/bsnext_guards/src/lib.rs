use crate::path_matcher::PathMatcher;
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::IntoResponse;
use http::Uri;

pub mod path_matcher;
pub mod root_path;
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

#[derive(Debug, Clone)]
pub struct OuterUri(pub Uri);
pub async fn uri_extension(uri: Uri, mut req: Request, next: Next) -> impl IntoResponse {
    req.extensions_mut().insert(OuterUri(uri));
    next.run(req).await
}
