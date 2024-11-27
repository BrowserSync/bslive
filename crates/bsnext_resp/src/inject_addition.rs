use crate::injector_guard::ByteReplacer;
use axum::extract::Request;
use bsnext_guards::route_guard::RouteGuard;
use http::Response;

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct InjectAddition {
    #[serde(flatten)]
    pub addition_position: AdditionPosition,
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum AdditionPosition {
    Append(String),
    Prepend(String),
}

impl RouteGuard for InjectAddition {
    fn accept_req(&self, _req: &Request) -> bool {
        true
    }

    fn accept_res<T>(&self, _res: &Response<T>) -> bool {
        true
    }
}
impl ByteReplacer for InjectAddition {
    fn apply(&self, body: &'_ str) -> Option<String> {
        match &self.addition_position {
            AdditionPosition::Append(content) => Some(format!("{body}{content}")),
            AdditionPosition::Prepend(content) => Some(format!("{content}{body}")),
        }
    }
}
