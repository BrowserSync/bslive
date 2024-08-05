use crate::connector::Connector;
use crate::debug::Debug;
use crate::injector_guard::{ByteReplacer, InjectorGuard};
use axum::extract::Request;
use http::Response;

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct BuiltinStringDef {
    pub name: BuiltinStrings,
}
#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub enum BuiltinStrings {
    #[serde(rename = "bslive:connector")]
    Connector,
    #[serde(rename = "bslive:debug")]
    Debug,
}

impl InjectorGuard for BuiltinStringDef {
    fn accept_req(&self, req: &Request) -> bool {
        match self.name {
            BuiltinStrings::Connector => Connector.accept_req(req),
            BuiltinStrings::Debug => Debug.accept_req(req),
        }
    }

    fn accept_res<T>(&self, res: &Response<T>) -> bool {
        match self.name {
            BuiltinStrings::Connector => Connector.accept_res(res),
            BuiltinStrings::Debug => Debug.accept_res(res),
        }
    }
}

impl ByteReplacer for BuiltinStringDef {
    fn apply(&self, body: &'_ str) -> Option<String> {
        match self.name {
            BuiltinStrings::Connector => Connector.apply(body),
            BuiltinStrings::Debug => Debug.apply(body),
        }
    }
}