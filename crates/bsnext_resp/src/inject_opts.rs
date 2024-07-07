use crate::builtin_strings::BuiltinStrings;
use crate::inject_defs::InjectDefinition;
use crate::injector_guard::{ByteReplacer, InjectorGuard};

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum InjectOpts {
    Bool(bool),
    Items(Vec<Injection>),
}

impl InjectOpts {
    pub fn injections(&self) -> Vec<Injection> {
        match self {
            InjectOpts::Bool(true) => {
                vec![Injection::BsLive(BuiltinStrings::Connector)]
            }
            InjectOpts::Bool(false) => {
                vec![]
            }
            InjectOpts::Items(items) if items.is_empty() => vec![],
            // todo: is this too expensive?
            InjectOpts::Items(items) => items.to_owned(),
        }
    }
}

impl Default for InjectOpts {
    fn default() -> Self {
        Self::Bool(true)
    }
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum Injection {
    BsLive(BuiltinStrings),
    UnknownNamed(String),
    Def(InjectDefinition),
}

impl InjectorGuard for Injection {}
impl ByteReplacer for Injection {
    fn apply(&self, body: &'_ str) -> Option<String> {
        match self {
            Injection::BsLive(strs) => strs.apply(body),
            Injection::UnknownNamed(_) => todo!("Injection::UnknownNamed"),
            Injection::Def(def) => def.apply(body),
        }
    }
}
