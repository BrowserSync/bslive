#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum InjectOpts {
    Bool(bool),
    Items(Vec<String>),
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
            InjectOpts::Items(_) => todo!("implement injection list"),
        }
    }
}

impl Default for InjectOpts {
    fn default() -> Self {
        Self::Bool(true)
    }
}
