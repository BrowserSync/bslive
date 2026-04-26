use crate::route::WatchSpec;

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum WatchOpts {
    Bool(bool),
    InlineGlob(String),
    Spec(WatchSpec),
}

impl Default for WatchOpts {
    fn default() -> Self {
        Self::Bool(true)
    }
}

impl WatchOpts {
    pub fn is_enabled(&self) -> bool {
        !matches!(self, WatchOpts::Bool(false))
    }
    pub fn spec(&self) -> Option<&WatchSpec> {
        match self {
            WatchOpts::Bool(_) => None,
            WatchOpts::InlineGlob(_) => None,
            WatchOpts::Spec(spec) => Some(spec),
        }
    }
}
