use crate::tasks::{append_with_reports, Comms, Index, Runnable, RunnableWithComms};
use crate::BsSystem;
use actix::Addr;
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_dto::archy::ArchyNode;
use bsnext_input::route::RunOptItem;
use bsnext_task::task_entry::TaskEntry;
use bsnext_task::task_report::TaskReport;
use bsnext_task::task_scope::TaskScope;
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
pub struct TaskSpec {
    pub run_kind: RunKind,
    pub tasks: Vec<Runnable>,
}

impl TreeDisplay for TaskSpec {
    fn as_tree_label(&self, Index(index): Index) -> String {
        let sqid = self.as_sqid(index);
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

impl TaskSpec {
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

impl TaskSpec {
    pub fn as_sqid(&self, parent: u64) -> String {
        let sqids = sqids::Sqids::default();
        let sqid = sqids.encode(&[parent]).unwrap();
        sqid.get(0..6).unwrap_or(&sqid).to_string()
    }
    pub fn as_tree(&self) -> ArchyNode {
        let label = self.as_tree_label(Index(0));
        let mut first = ArchyNode::new(&label);
        let empty = HashMap::default();
        append_with_reports(&mut first, &self.tasks, &empty);
        first
    }
    pub fn as_tree_with_results(&self, hm: &HashMap<u64, TaskReport>) -> ArchyNode {
        // let label = self.to_string();
        let r = hm.get(&self.as_id());
        let label = match r {
            None => "missing".to_string(),
            Some(_) => self.as_tree_label(Index(0)),
        };
        let mut first = ArchyNode::new(&label);
        append_with_reports(&mut first, &self.tasks, hm);
        first
    }
}

impl TreeDisplay for Runnable {
    fn as_tree_label(&self, index: Index) -> String {
        match self {
            Runnable::BsLiveTask(item) => format!("{}{}", "Runnable::BsLiveTask", item),
            Runnable::Sh(sh) => format!("{} {}", "Runnable::Sh", sh),
            Runnable::Many(task_spec) => {
                let id = self.as_id_with(index);
                task_spec.as_tree_label(Index(id))
            }
        }
    }
}

pub trait TreeDisplay {
    fn as_tree_label(&self, index: Index) -> String;
}

impl TaskSpec {
    pub fn to_task_scope(
        self,
        servers_addr: Option<Addr<ServersSupervisor>>,
        sys_addr: Addr<BsSystem>,
    ) -> TaskScope {
        let parent_id = self.as_id();
        let inner_tasks = self
            .tasks
            .into_iter()
            .enumerate()
            .map(|(index_position, runnable)| -> TaskEntry {
                let item_id = runnable.as_id_with(Index(index_position as u64));
                match runnable {
                    Runnable::Many(task_spec) => TaskEntry::new(
                        Box::new(task_spec.to_task_scope(servers_addr.clone(), sys_addr.clone())),
                        item_id,
                    ),
                    _ => {
                        let with_ctx = RunnableWithComms {
                            ctx: Comms {
                                servers_addr: servers_addr.clone(),
                                sys: sys_addr.clone(),
                            },
                            runnable,
                        };
                        TaskEntry::new(Box::new(with_ctx), item_id)
                    }
                }
            })
            .collect::<Vec<TaskEntry>>();
        match self.run_kind {
            RunKind::Sequence { opts } => TaskScope::seq(inner_tasks, opts, parent_id),
            RunKind::Overlapping { opts } => TaskScope::all(inner_tasks, opts, parent_id),
        }
    }
}
