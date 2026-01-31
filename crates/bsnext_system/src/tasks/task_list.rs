use crate::tasks::bs_live_task::BsLiveTask;
use crate::tasks::sh_cmd::ShCmd;
use crate::tasks::{append, append_with_reports, Runnable};
use bsnext_dto::archy::ArchyNode;
use bsnext_dto::internal::TaskReport;
use bsnext_input::route::{BsLiveRunner, RunAll, RunOptItem, RunSeq};
use bsnext_task::task_entry::TaskEntry;
use bsnext_task::task_group::TaskGroup;
use bsnext_task::{OverlappingOpts, RunKind, SequenceOpts};
use std::collections::HashMap;
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
        let sqid = self.as_sqid(parent);
        match &self.run_kind {
            RunKind::Sequence { .. } => format!("[{sqid}] Seq: {} task(s)", self.tasks.len()),
            RunKind::Overlapping { opts } => format!(
                "[{sqid}] Overlapping {} task(s) (max concurrency: {})",
                self.tasks.len(),
                opts.max_concurrent_items
            ),
        }
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
    pub fn seq_opts(p0: &[Runnable], opts: SequenceOpts) -> Self {
        Self {
            run_kind: RunKind::Sequence { opts },
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
    pub fn as_sqid(&self, parent: u64) -> String {
        let sqids = sqids::Sqids::default();
        let sqid = sqids.encode(&[parent]).unwrap();
        sqid.get(0..6).unwrap_or(&sqid).to_string()
    }
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
    pub fn as_sqid(&self, id: u64) -> String {
        let sqids = sqids::Sqids::default();
        let sqid = sqids.encode(&[id]).unwrap_or_else(|_| id.to_string());
        sqid.get(0..6).map(String::from).unwrap_or(sqid)
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
                BsLiveRunner::PublishExternalEvent => {
                    Self::BsLiveTask(BsLiveTask::PublishExternalEvent)
                }
            },
            RunOptItem::Sh(sh) => Self::Sh(ShCmd::from(sh)),
            RunOptItem::ShImplicit(sh) => Self::Sh(ShCmd::new(sh.into())),
            RunOptItem::All(RunAll { all, run_all_opts }) => {
                let items: Vec<_> = all.iter().map(Runnable::from).collect();
                let opts = OverlappingOpts {
                    max_concurrent_items: run_all_opts.max,
                    exit_on_failure: run_all_opts.exit_on_fail,
                };
                Self::Many(TaskList::all(&items, opts))
            }
            RunOptItem::Seq(RunSeq { seq, seq_opts }) => {
                let items: Vec<_> = seq.iter().map(Runnable::from).collect();
                let opts = SequenceOpts {
                    exit_on_failure: seq_opts.exit_on_fail,
                };
                Self::Many(TaskList::seq_opts(&items, opts))
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

impl From<TaskList> for TaskGroup {
    fn from(runner: TaskList) -> Self {
        let top_id = runner.as_id();
        let boxed_tasks = runner
            .tasks
            .into_iter()
            .enumerate()
            .map(|(i, x)| -> TaskEntry {
                let item_id = x.as_id_with(i as u64);
                match x {
                    Runnable::Many(runner) => {
                        TaskEntry::new(Box::new(TaskGroup::from(runner)), item_id)
                    }
                    _ => TaskEntry::new(Box::new(x), item_id),
                }
            })
            .collect::<Vec<TaskEntry>>();
        match runner.run_kind {
            RunKind::Sequence { opts } => Self::seq(boxed_tasks, opts, top_id),
            RunKind::Overlapping { opts } => Self::all(boxed_tasks, opts, top_id),
        }
    }
}
