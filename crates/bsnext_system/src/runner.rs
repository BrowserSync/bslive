use crate::task::{AsActor, TaskCommand};
use crate::tasks::notify_servers::NotifyServers;
use crate::tasks::sh_cmd::ShCmd;
use actix::{Actor, Recipient};
use bsnext_input::route::{BsLiveRunner, RunOptItem};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct Runner {
    pub kind: RunKind,
    pub tasks: Vec<Runnable>,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum RunKind {
    Sequence,
    Overlapping,
}

impl Runner {
    pub fn all(p0: &[Runnable]) -> Self {
        Self {
            kind: RunKind::Overlapping,
            tasks: p0.to_vec(),
        }
    }
    pub fn seq(p0: &[Runnable]) -> Self {
        Self {
            kind: RunKind::Sequence,
            tasks: p0.to_vec(),
        }
    }
    pub fn all_from(p0: &[RunOptItem]) -> Self {
        Self {
            kind: RunKind::Overlapping,
            tasks: p0.into_iter().map(|opt| Runnable::from(opt)).collect(),
        }
    }
    pub fn seq_from(p0: &[RunOptItem]) -> Self {
        Self {
            kind: RunKind::Sequence,
            tasks: p0.into_iter().map(|opt| Runnable::from(opt)).collect(),
        }
    }
}

impl AsActor for Runnable {
    fn into_actor2(self: Box<Self>) -> Recipient<TaskCommand> {
        match *self {
            Runnable::BsLive(BsLiveRunner::NotifyServer) => {
                let s = NotifyServers::new();
                let s = s.start();
                s.recipient()
            }
            Runnable::Sh(sh) => {
                let s = sh.start();
                s.recipient()
            }
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum Runnable {
    BsLive(BsLiveRunner),
    Sh(ShCmd),
}

impl From<&RunOptItem> for Runnable {
    fn from(value: &RunOptItem) -> Self {
        match value {
            RunOptItem::BsLive { bslive } => Self::BsLive(bslive.clone()),
            RunOptItem::Sh { sh } | RunOptItem::ShImplicit(sh) => Self::Sh(ShCmd::new(sh.into())),
        }
    }
}

impl From<RunOptItem> for Runnable {
    fn from(value: RunOptItem) -> Self {
        Runnable::from(&value)
    }
}
