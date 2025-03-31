use crate::task::{AsActor, TaskCommand, TreePath};
use crate::tasks::notify_servers::NotifyServers;
use crate::tasks::sh_cmd::ShCmd;
use actix::{Actor, Recipient};
use bsnext_input::route::{BsLiveRunner, RunOptItem};
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct Runner {
    pub kind: RunKind,
    pub tasks: Vec<Runnable>,
}

impl TreePath for Runner {
    fn append(&self, parents: &mut Vec<u64>) {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let h = hasher.finish();
        parents.push(h);
    }
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

    pub fn add(&mut self, r: Runnable) {
        self.tasks.push(r);
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
            Runnable::Many(_) => unreachable!("The conversion to Task happens elsewhere"),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum Runnable {
    BsLive(BsLiveRunner),
    Sh(ShCmd),
    Many(Runner),
}

impl TreePath for Runnable {
    fn append(&self, parents: &mut Vec<u64>) {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        let h = hasher.finish();
        parents.push(h);
    }
}

impl From<&RunOptItem> for Runnable {
    fn from(value: &RunOptItem) -> Self {
        match value {
            RunOptItem::BsLive { bslive } => Self::BsLive(bslive.clone()),
            RunOptItem::Sh { sh } | RunOptItem::ShImplicit(sh) => Self::Sh(ShCmd::new(sh.into())),
            RunOptItem::All { all } => {
                let items: Vec<_> = all.iter().map(|x| Runnable::from(x)).collect();
                Self::Many(Runner::all(&items))
            }
            RunOptItem::Seq { seq } => {
                let items: Vec<_> = seq.iter().map(|x| Runnable::from(x)).collect();
                Self::Many(Runner::seq(&items))
            }
        }
    }
}

impl From<RunOptItem> for Runnable {
    fn from(value: RunOptItem) -> Self {
        Runnable::from(&value)
    }
}
