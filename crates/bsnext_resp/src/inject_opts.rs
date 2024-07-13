use crate::builtin_strings::{BuiltinStringDef, BuiltinStrings};
use crate::inject_addition::InjectAddition;
use crate::inject_replacement::InjectReplacement;
use crate::injector_guard::{ByteReplacer, InjectorGuard};
use axum::extract::Request;
use http::Response;

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum InjectOpts {
    Bool(bool),
    Item(InjectionItem),
    Items(Vec<InjectionItem>),
}

impl InjectOpts {
    pub fn injections(&self) -> Vec<InjectionItem> {
        match self {
            InjectOpts::Bool(true) => {
                vec![InjectionItem {
                    inner: Injection::BsLive(BuiltinStringDef {
                        name: BuiltinStrings::Connector,
                    }),
                }]
            }
            InjectOpts::Bool(false) => {
                vec![]
            }
            InjectOpts::Items(items) if items.is_empty() => vec![],
            // todo: is this too expensive?
            InjectOpts::Items(items) => items.to_owned(),
            InjectOpts::Item(item) => vec![item.to_owned()],
        }
    }
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

impl InjectorGuard for InjectionItem {
    fn accept_req(&self, req: &Request) -> bool {
        match &self.inner {
            Injection::BsLive(built_ins) => built_ins.accept_req(req),
            Injection::UnknownNamed(_) => todo!("accept_req Injection::UnknownNamed"),
            Injection::Replacement(def) => def.accept_req(req),
            Injection::Addition(add) => add.accept_req(req),
        }
    }

    fn accept_res<T>(&self, res: &Response<T>) -> bool {
        match &self.inner {
            Injection::BsLive(built_ins) => built_ins.accept_res(res),
            Injection::UnknownNamed(_) => todo!("accept_res Injection::UnknownNamed"),
            Injection::Replacement(def) => def.accept_res(res),
            Injection::Addition(add) => add.accept_res(res),
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

// impl InjectorGuard for Injection {
//     fn accept_req(&self, req: &Request) -> bool {
//         match self {
//             Injection::BsLive(built_ins) => built_ins.accept_req(req),
//             Injection::UnknownNamed(_) => todo!("accept_req Injection::UnknownNamed"),
//             Injection::Replacement(def) => def.accept_req(req),
//             Injection::Addition(add) => add.accept_req(req),
//         }
//     }
//
//     fn accept_res<T>(&self, res: &Response<T>) -> bool {
//         match self {
//             Injection::BsLive(built_ins) => built_ins.accept_res(res),
//             Injection::UnknownNamed(_) => todo!("accept_res Injection::UnknownNamed"),
//             Injection::Replacement(def) => def.accept_res(res),
//             Injection::Addition(add) => add.accept_res(res),
//         }
//     }
// }
// impl ByteReplacer for Injection {
//     fn apply(&self, body: &'_ str) -> Option<String> {
//         match self {
//             Injection::BsLive(strs) => strs.apply(body),
//             Injection::UnknownNamed(_) => todo!("Injection::UnknownNamed"),
//             Injection::Replacement(def) => def.apply(body),
//             Injection::Addition(add) => add.apply(body),
//         }
//     }
// }
