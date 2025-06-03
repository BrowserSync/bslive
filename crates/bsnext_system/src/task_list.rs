use crate::ext_event_sender::ExtEventSender;
use crate::task::{AsActor, TaskCommand};
use crate::tasks::notify_servers::NotifyServers;
use crate::tasks::sh_cmd::ShCmd;
use actix::{Actor, Recipient};
use bsnext_dto::archy::ArchyNode;
use bsnext_dto::internal::TaskReport;
use bsnext_input::route::{BsLiveRunner, RunAll, RunOptItem, RunSeq};
use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};

/// Represents a collection of tasks that can be run, categorized by their execution type (`RunKind`).
///
/// This `TaskList` struct provides a way to organize and manage a collection of runnable tasks.
/// Each task is encapsulated within the `Runnable` type, and the execution behavior of the task list is determined
/// by the `RunKind`.
///
/// # Fields
///
/// * `run_kind`:
///   Specifies the type of execution behavior (defined by the [`RunKind`] enum) for the task list.
///
/// * `tasks`:
///   A vector containing the individual tasks to be executed. Each task is represented as an instance of the `Runnable` struct.
///
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct TaskList {
    pub run_kind: RunKind,
    pub tasks: Vec<Runnable>,
}

impl TreeDisplay for TaskList {
    fn as_tree_label(&self, parent: u64) -> String {
        let _id = self.as_id_with(parent);
        match &self.run_kind {
            RunKind::Sequence { .. } => format!("Seq: {} task(s)", self.tasks.len()),
            RunKind::Overlapping { opts } => format!(
                "Overlapping {} task(s) (max concurrency: {})",
                self.tasks.len(),
                opts.max_concurrent_items
            ),
        }
    }
}

/// The `RunKind` enum represents the type of execution or arrangement of a set of operations or elements.
/// It provides two distinct variants: `Sequence` and `Overlapping`.
///
/// ## Variants
///
/// - `Sequence`:
///   Represents a straightforward sequential arrangement or execution.
///   Operations or elements will proceed one after another in the specified order.
///
/// - `Overlapping`:
///   Represents an overlapping arrangement where operations or elements can overlap or run concurrently,
///   based on specific options provided.
///
///   - `opts`: A field of type `OverlappingOpts` that contains the configuration or parameters
///     dictating the behavior of overlapping operations.
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum RunKind {
    Sequence { opts: SequenceOpts },
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
#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct SequenceOpts {
    pub exit_on_failure: bool,
}

impl Default for SequenceOpts {
    fn default() -> Self {
        Self {
            exit_on_failure: true,
        }
    }
}

impl SequenceOpts {
    pub fn new(exit_on_failure: bool) -> Self {
        Self { exit_on_failure }
    }
}

impl TaskList {
    pub fn all(p0: &[Runnable], opts: OverlappingOpts) -> Self {
        Self {
            run_kind: RunKind::Overlapping { opts },
            tasks: p0.to_vec(),
        }
    }
    pub fn seq(p0: &[Runnable]) -> Self {
        Self {
            run_kind: RunKind::Sequence {
                opts: SequenceOpts::default(),
            },
            tasks: p0.to_vec(),
        }
    }
    pub fn seq_from(p0: &[RunOptItem]) -> Self {
        Self {
            run_kind: RunKind::Sequence {
                opts: SequenceOpts::default(),
            },
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

impl TaskList {
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
            Runnable::BsLiveTask(_) => archy.nodes.push(ArchyNode::new(&label)),
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
            Runnable::BsLiveTask(_) => archy.nodes.push(ArchyNode::new(&label)),
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
    fn into_task_recipient(self: Box<Self>) -> Recipient<TaskCommand> {
        match *self {
            Runnable::BsLiveTask(BsLiveTask::NotifyServer) => {
                let s = NotifyServers::new();
                let s = s.start();
                s.recipient()
            }
            Runnable::BsLiveTask(BsLiveTask::ExtEvent) => {
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
    BsLiveTask(BsLiveTask),
    Sh(ShCmd),
    Many(TaskList),
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum BsLiveTask {
    NotifyServer,
    ExtEvent,
}

impl Display for BsLiveTask {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BsLiveTask::NotifyServer => write!(f, "BsLiveTask::NotifyServer"),
            BsLiveTask::ExtEvent => write!(f, "BsLiveTask::Group"),
        }
    }
}

impl Runnable {
    pub fn is_group(&self) -> bool {
        match self {
            Runnable::BsLiveTask(_) => false,
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
            Runnable::BsLiveTask(item) => format!("{}{}", "Runnable::BsLiveTask", item),
            Runnable::Sh(sh) => format!("{} {}", "Runnable::Sh", sh),
            Runnable::Many(runner) => runner.as_tree_label(id),
        }
    }
}

impl From<&RunOptItem> for Runnable {
    fn from(value: &RunOptItem) -> Self {
        match value {
            RunOptItem::BsLive { bslive } => match bslive {
                BsLiveRunner::NotifyServer => Self::BsLiveTask(BsLiveTask::NotifyServer),
                BsLiveRunner::ExtEvent => Self::BsLiveTask(BsLiveTask::ExtEvent),
            },
            RunOptItem::Sh(sh) => Self::Sh(ShCmd::from(sh)),
            RunOptItem::ShImplicit(sh) => Self::Sh(ShCmd::new(sh.into())),
            RunOptItem::All(RunAll { all, run_all_opts }) => {
                let items: Vec<_> = all.iter().map(Runnable::from).collect();
                let opts = OverlappingOpts {
                    max_concurrent_items: run_all_opts.max,
                };
                Self::Many(TaskList::all(&items, opts))
            }
            RunOptItem::Seq(RunSeq { seq, .. }) => {
                let items: Vec<_> = seq.iter().map(Runnable::from).collect();
                Self::Many(TaskList::seq(&items))
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
