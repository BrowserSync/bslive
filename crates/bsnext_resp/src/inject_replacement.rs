use crate::injector_guard::{ByteReplacer, InjectorGuard};
use axum::extract::Request;
use http::Response;

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct InjectReplacement {
    #[serde(flatten)]
    pub pos: Pos,
    pub content: String,
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Pos {
    Before(String),
    After(String),
    Replace(String),
}

impl InjectorGuard for InjectReplacement {
    fn accept_req(&self, _req: &Request) -> bool {
        true
    }

    fn accept_res<T>(&self, _res: &Response<T>) -> bool {
        true
    }
}
impl ByteReplacer for InjectReplacement {
    fn apply(&self, body: &'_ str) -> Option<String> {
        match &self.pos {
            Pos::Before(matcher) => {
                Some(body.replace(matcher, &format!("{}{}", &self.content, matcher)))
            }
            Pos::After(matcher) => {
                Some(body.replace(matcher, &format!("{}{}", matcher, &self.content)))
            }
            Pos::Replace(matcher) => Some(body.replace(matcher, &self.content)),
        }
    }
}
