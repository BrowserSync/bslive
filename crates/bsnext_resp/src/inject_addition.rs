use crate::injector_guard::{ByteReplacer, InjectorGuard};
use crate::RespMod;
use axum::extract::Request;
use http::Response;

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct InjectAddition {
    #[serde(flatten)]
    pub end: AdditionPosition,
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AdditionPosition {
    Append(String),
    Prepend(String),
}

impl InjectorGuard for InjectAddition {
    fn accept_req(&self, req: &Request) -> bool {
        RespMod::accepts_html(req)
    }

    fn accept_res<T>(&self, res: &Response<T>) -> bool {
        RespMod::is_html(res)
    }
}
impl ByteReplacer for InjectAddition {
    fn apply(&self, body: &'_ str) -> Option<String> {
        match &self.end {
            AdditionPosition::Append(content) => Some(format!("{body}{content}")),
            AdditionPosition::Prepend(content) => Some(format!("{content}{body}")),
        }
    }
}
