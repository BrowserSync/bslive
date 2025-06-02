use crate::ext_event_sender::ExtEventSender;
use crate::task::{AsActor, TaskCommand};
use crate::tasks::notify_servers::NotifyServers;
use crate::tasks::sh_cmd::ShCmd;
use actix::{Actor, Recipient};
use bsnext_dto::archy::ArchyNode;
use bsnext_dto::internal::TaskReport;
use bsnext_input::route::{BsLiveRunner, RunAll, RunOptItem, RunSeq};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct Runner {
    pub run_kind: RunKind,
    pub tasks: Vec<Runnable>,
}

impl TreeDisplay for Runner {
    fn as_tree_label(&self, parent: u64) -> String {
        let _id = self.as_id_with(parent);
        match &self.run_kind {
            RunKind::Sequence => format!("Seq: {} task(s)", self.tasks.len()),
            RunKind::Overlapping { .. } => format!("Overlapping {} task(s)", self.tasks.len()),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum RunKind {
    Sequence,
    Overlapping { opts: OverlappingOpts },
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct OverlappingOpts {
    pub max_concurrent_items: u8,
}

impl OverlappingOpts {
    pub fn new(max_concurrent_items: u8) -> Self {
        Self {
            max_concurrent_items,
        }
    }
}

impl Runner {
    pub fn all(p0: &[Runnable], opts: OverlappingOpts) -> Self {
        Self {
            run_kind: RunKind::Overlapping { opts },
            tasks: p0.to_vec(),
        }
    }
    pub fn seq(p0: &[Runnable]) -> Self {
        Self {
            run_kind: RunKind::Sequence,
            tasks: p0.to_vec(),
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
    pub fn as_id_with(&self, parent: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        parent.hash(&mut hasher);
        hasher.finish()
    }
}

impl Runner {
    pub fn as_tree(&self) -> ArchyNode {
        let label = self.as_tree_label(0);
        let mut first = ArchyNode::new(&label);
        append(&mut first, &self.tasks);
        first
    }
    pub fn as_tree_with_results(&self, hm: &HashMap<u64, TaskReport>) -> ArchyNode {
        // let label = self.to_string();
        let r = hm.get(&self.as_id());
        let label = match r {
            None => "missing".to_string(),
            Some(_) => self.as_tree_label(0),
        };
        let mut first = ArchyNode::new(&label);
        append_with_reports(&mut first, &self.tasks, hm);
        first
    }
}

fn append(archy: &mut ArchyNode, tasks: &[Runnable]) {
    for (i, x) in tasks.iter().enumerate() {
        let label = x.as_tree_label(i as u64);
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
    for (i, runnable) in tasks.iter().enumerate() {
        let id = runnable.as_id_with(i as u64);
        let label = match hm.get(&id) {
            None => format!("− {}", runnable.as_tree_label(i as u64)),
            Some(report) => {
                if runnable.is_group() {
                    runnable.as_tree_label(i as u64)
                } else {
                    format!(
                        "{} {}",
                        if report.is_ok() { "✅" } else { "❌" },
                        runnable.as_tree_label(i as u64)
                    )
                }
            }
        };
        match runnable {
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
    pub fn is_group(&self) -> bool {
        match self {
            Runnable::BsLive(_) => false,
            Runnable::Sh(_) => false,
            Runnable::Many(_) => true,
        }
    }
    pub fn as_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
    pub fn as_id_with(&self, parent: u64) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        parent.hash(&mut hasher);
        hasher.finish()
    }
}

impl TreeDisplay for Runnable {
    fn as_tree_label(&self, parent: u64) -> String {
        let id = self.as_id_with(parent);
        match self {
            Runnable::BsLive(item) => format!("{}{}", "Runnable::BsLive::", item),
            Runnable::Sh(sh) => format!("{} {}", "Runnable::Sh", sh),
            Runnable::Many(runner) => runner.as_tree_label(id),
        }
    }
}

impl From<&RunOptItem> for Runnable {
    fn from(value: &RunOptItem) -> Self {
        match value {
            RunOptItem::BsLive { bslive } => Self::BsLive(bslive.clone()),
            RunOptItem::Sh(sh) => Self::Sh(ShCmd::from(sh)),
            RunOptItem::ShImplicit(sh) => Self::Sh(ShCmd::new(sh.into())),
            RunOptItem::All(RunAll { all, run_all_opts }) => {
                let items: Vec<_> = all.iter().map(Runnable::from).collect();
                let opts = OverlappingOpts {
                    max_concurrent_items: run_all_opts.max,
                };
                Self::Many(Runner::all(&items, opts))
            }
            RunOptItem::Seq(RunSeq { seq, .. }) => {
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

pub trait TreeDisplay {
    fn as_tree_label(&self, parent: u64) -> String;
}
