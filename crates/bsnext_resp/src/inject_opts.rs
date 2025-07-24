use crate::builtin_strings::BuiltinStringDef;
use crate::inject_addition::InjectAddition;
use crate::inject_replacement::InjectReplacement;
use crate::injector_guard::ByteReplacer;
use axum::extract::Request;
use bsnext_guards::route_guard::RouteGuard;
use bsnext_guards::MatcherList;
use http::{Response, Uri};

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum InjectOpts {
    Bool(bool),
    Item(InjectionItem),
    Items(Vec<InjectionItem>),
}

impl Default for InjectOpts {
    fn default() -> Self {
        Self::Bool(true)
    }
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct InjectionItem {
    #[serde(flatten)]
    pub inner: Injection,
    pub only: Option<MatcherList>,
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum Injection {
    BsLive(BuiltinStringDef),
    UnknownNamed(UnknownStringDef),
    Replacement(InjectReplacement),
    Addition(InjectAddition),
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct UnknownStringDef {
    pub name: String,
}
impl RouteGuard for InjectionItem {
    fn accept_req(&self, req: &Request, outer_uri: &Uri) -> bool {
        let uri_is_allowed = match self.only.as_ref() {
            None => true,
            Some(ml) => ml.test_uri(outer_uri),
        };
        if uri_is_allowed {
            match &self.inner {
                Injection::BsLive(built_ins) => built_ins.accept_req(req, outer_uri),
                Injection::UnknownNamed(_) => todo!("accept_req Injection::UnknownNamed"),
                Injection::Replacement(def) => def.accept_req(req, outer_uri),
                Injection::Addition(add) => add.accept_req(req, outer_uri),
            }
        } else {
            false
        }
    }

    fn accept_res<T>(&self, res: &Response<T>, outer_uri: &Uri) -> bool {
        match &self.inner {
            Injection::BsLive(built_ins) => built_ins.accept_res(res, outer_uri),
            Injection::UnknownNamed(_) => todo!("accept_res Injection::UnknownNamed"),
            Injection::Replacement(def) => def.accept_res(res, outer_uri),
            Injection::Addition(add) => add.accept_res(res, outer_uri),
        }
    }
}
impl ByteReplacer for InjectionItem {
    fn apply(&self, body: &'_ str) -> Option<String> {
        match &self.inner {
            Injection::BsLive(strs) => strs.apply(body),
            Injection::UnknownNamed(_) => todo!("Injection::UnknownNamed"),
            Injection::Replacement(def) => def.apply(body),
            Injection::Addition(add) => add.apply(body),
        }
    }
}
