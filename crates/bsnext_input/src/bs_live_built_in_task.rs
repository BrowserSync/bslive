use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[derive(
    Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone, serde::Deserialize, serde::Serialize,
)]
pub enum BsLiveBuiltInTask {
    #[serde(rename = "notify-server")]
    NotifyServer,
    #[serde(rename = "ext-event")]
    PublishExternalEvent,
}

impl FromStr for BsLiveBuiltInTask {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "notify-server" => Ok(Self::NotifyServer),
            "ext-event" => Ok(Self::PublishExternalEvent),
            _ => Err(anyhow::anyhow!("not a valid bslive builtin task")),
        }
    }
}

impl Display for BsLiveBuiltInTask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BsLiveBuiltInTask::NotifyServer => write!(f, "BsLiveTask::NotifyServer"),
            BsLiveBuiltInTask::PublishExternalEvent => {
                write!(f, "BsLiveTask::PublishExternalEvent")
            }
        }
    }
}
