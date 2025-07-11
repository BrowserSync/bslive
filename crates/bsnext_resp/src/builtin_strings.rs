use crate::connector::Connector;
use crate::debug::Debug;
use crate::injector_guard::ByteReplacer;
use crate::js_connector::JsConnector;
use axum::extract::Request;
use bsnext_guards::route_guard::RouteGuard;
use http::{Response, Uri};

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct BuiltinStringDef {
    pub name: BuiltinStrings,
}
#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub enum BuiltinStrings {
    #[serde(rename = "bslive:connector")]
    Connector,
    #[serde(rename = "bslive:js-connector")]
    JsConnector,
    #[serde(rename = "bslive:debug")]
    Debug,
}

impl RouteGuard for BuiltinStringDef {
    fn accept_req(&self, req: &Request, outer_uri: &Uri) -> bool {
        match self.name {
            BuiltinStrings::Connector => Connector.accept_req(req, outer_uri),
            BuiltinStrings::Debug => Debug.accept_req(req, outer_uri),
            BuiltinStrings::JsConnector => JsConnector.accept_req(req, outer_uri),
        }
    }

    fn accept_res<T>(&self, res: &Response<T>, outer_uri: &Uri) -> bool {
        match self.name {
            BuiltinStrings::Connector => Connector.accept_res(res, outer_uri),
            BuiltinStrings::Debug => Debug.accept_res(res, outer_uri),
            BuiltinStrings::JsConnector => JsConnector.accept_res(res, outer_uri),
        }
    }
}

impl ByteReplacer for BuiltinStringDef {
    fn apply(&self, body: &'_ str) -> Option<String> {
        match self.name {
            BuiltinStrings::Connector => Connector.apply(body),
            BuiltinStrings::Debug => Debug.apply(body),
            BuiltinStrings::JsConnector => JsConnector.apply(body),
        }
    }
}
