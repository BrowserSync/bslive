use crate::archy::ArchyNode;
use crate::ext_event_sender::ExtEventSender;
use crate::task::{AsActor, Task, TaskCommand, TaskReport, TaskResult};
use crate::task_group::TaskGroup;
use crate::tasks::notify_servers::NotifyServers;
use crate::tasks::sh_cmd::ShCmd;
use actix::{Actor, Recipient};
use bsnext_input::route::{BsLiveRunner, RunOptItem};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct Runner {
    pub run_kind: RunKind,
    pub tasks: Vec<Runnable>,
}

impl Display for Runner {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let id = self.as_id();
        match &self.run_kind {
            RunKind::Sequence => write!(f, "Seq: {} task(s) {}", self.tasks.len(), id),
            RunKind::Overlapping => write!(f, "Overlapping {} task(s) {}", self.tasks.len(), id),
        }
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
            run_kind: RunKind::Overlapping,
            tasks: p0.to_vec(),
        }
    }
    pub fn seq(p0: &[Runnable]) -> Self {
        Self {
            run_kind: RunKind::Sequence,
            tasks: p0.to_vec(),
        }
    }
    pub fn all_from(p0: &[RunOptItem]) -> Self {
        Self {
            run_kind: RunKind::Overlapping,
            tasks: p0.iter().map(Runnable::from).collect(),
        }
    }
    pub fn seq_from(p0: &[RunOptItem]) -> Self {
        Self {
            run_kind: RunKind::Sequence,
            tasks: p0.iter().map(Runnable::from).collect(),
        }
    }

    pub fn add(&mut self, r: Runnable) {
        self.tasks.push(r);
    }

    pub fn as_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl Runner {
    pub fn as_tree(&self) -> ArchyNode {
        let label = self.to_string();
        let mut first = ArchyNode::new(&label);
        append(&mut first, &self.tasks);
        first
    }
    pub fn as_tree_with_results(&self, hm: &HashMap<u64, TaskReport>) -> ArchyNode {
        // let label = self.to_string();
        let r = hm.get(&self.as_id());
        let label = match r {
            None => "missing".to_string(),
            Some(item) => {
                let good = item.result().is_ok();
                format!("{} {}", if good { "✅" } else { "❌" }, self.to_string())
            }
        };
        let mut first = ArchyNode::new(&label);
        append_with_reports(&mut first, &self.tasks, &hm);
        first
    }
}

fn append(archy: &mut ArchyNode, tasks: &[Runnable]) {
    for x in tasks {
        let label = x.to_string();
        match x {
            Runnable::BsLive(_) => archy.nodes.push(ArchyNode::new(&label)),
            Runnable::Sh(_) => archy.nodes.push(ArchyNode::new(&label)),
            Runnable::Many(runner) => {
                let mut next = ArchyNode::new(&label);
                append(&mut next, &runner.tasks);
                archy.nodes.push(next);
            }
        }
    }
}

fn append_with_reports(archy: &mut ArchyNode, tasks: &[Runnable], hm: &HashMap<u64, TaskReport>) {
    for x in tasks {
        let id = x.as_id();
        let label = match hm.get(&id) {
            None => x.to_string(),
            Some(report) => format!(
                "{} {}",
                if report.is_ok() { "✅" } else { "❌" },
                x.to_string()
            ),
        };
        match x {
            Runnable::BsLive(_) => archy.nodes.push(ArchyNode::new(&label)),
            Runnable::Sh(_) => archy.nodes.push(ArchyNode::new(&label)),
            Runnable::Many(runner) => {
                let mut next = ArchyNode::new(&label);
                append_with_reports(&mut next, &runner.tasks, hm);
                archy.nodes.push(next);
            }
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
            Runnable::BsLive(BsLiveRunner::ExtEvent) => {
                let actor = ExtEventSender::new();
                let addr = actor.start();
                addr.recipient()
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

impl Runnable {
    pub fn as_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

impl Display for Runnable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let id = self.as_id();
        match self {
            Runnable::BsLive(item) => write!(f, "{}{} {}", "Runnable::BsLive::", item, id),
            Runnable::Sh(sh) => write!(f, "{} {} {}", "Runnable::Sh", sh, id),
            Runnable::Many(runner) => write!(f, "{} {}", runner, id),
        }
    }
}

impl TreeDisplay for Runnable {
    fn as_tree_label(&self, res: &TaskResult) -> String {
        self.to_string()
    }
}

impl From<&RunOptItem> for Runnable {
    fn from(value: &RunOptItem) -> Self {
        match value {
            RunOptItem::BsLive { bslive } => Self::BsLive(bslive.clone()),
            RunOptItem::Sh(sh) => Self::Sh(ShCmd::from(sh)),
            RunOptItem::ShImplicit(sh) => Self::Sh(ShCmd::new(sh.into())),
            RunOptItem::All { all } => {
                let items: Vec<_> = all.iter().map(Runnable::from).collect();
                Self::Many(Runner::all(&items))
            }
            RunOptItem::Seq { seq } => {
                let items: Vec<_> = seq.iter().map(Runnable::from).collect();
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

pub trait TreeDisplay: std::fmt::Display {
    fn as_tree_label(&self, res: &TaskResult) -> String {
        self.to_string()
    }
}
