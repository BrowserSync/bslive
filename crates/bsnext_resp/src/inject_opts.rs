#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum InjectOpts {
    Bool(bool),
    Items(Vec<Known>),
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum Known {
    BsLive(BsLiveStrings),
    UnknownNamed(String),
    Def(InjectDefinition),
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub enum BsLiveStrings {
    #[serde(rename = "bslive:connector")]
    Connector,
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub struct InjectDefinition {
    pub name: String,
    pub before: String,
    pub content: String,
}

impl InjectOpts {
    pub fn injections(&self) -> Vec<String> {
        match self {
            InjectOpts::Bool(true) => {
                vec!["bslive:connector".to_string()]
            }
            InjectOpts::Bool(false) => {
                vec![]
            }
            InjectOpts::Items(items) if items.is_empty() => vec![],
            InjectOpts::Items(items) => todo!("implement vec list"),
        }
    }
}

impl Default for InjectOpts {
    fn default() -> Self {
        Self::Bool(true)
    }
}
