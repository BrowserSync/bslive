use crate::capabilities::Capabilities;
use crate::tasks::comms::Comms;
use crate::tasks::{Runnable, RunnableWithComms, Sqid};
use actix::Addr;
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_dto::archy::ArchyNode;
use bsnext_input::route::RunOptItem;
use bsnext_task::invocation::SpecId;
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
    run_kind: RunKind,
    tasks: Vec<Runnable>,
}

impl TreeDisplay for TaskSpec {
    fn as_tree_label(&self) -> String {
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

impl TaskSpec {
    pub fn is_seq(&self) -> bool {
        matches!(self.run_kind, RunKind::Sequence { .. })
    }
    pub fn is_all(&self) -> bool {
        matches!(self.run_kind, RunKind::Overlapping { .. })
    }
    pub fn all(tasks: &[Runnable], opts: OverlappingOpts) -> Self {
        Self {
            run_kind: RunKind::Overlapping { opts },
            tasks: tasks.to_vec(),
        }
    }
    pub fn seq(tasks: &[Runnable]) -> Self {
        Self {
            run_kind: RunKind::Sequence {
                opts: SequenceOpts::default(),
            },
            tasks: tasks.to_vec(),
        }
    }
    pub fn seq_opts(tasks: &[Runnable], opts: SequenceOpts) -> Self {
        Self {
            run_kind: RunKind::Sequence { opts },
            tasks: tasks.to_vec(),
        }
    }
    pub fn seq_from(run_items: &[RunOptItem]) -> Self {
        Self {
            run_kind: RunKind::Sequence {
                opts: SequenceOpts::default(),
            },
            tasks: run_items.iter().map(Runnable::from).collect(),
        }
    }
    pub fn all_from(run_items: &[RunOptItem]) -> Self {
        Self {
            run_kind: RunKind::Overlapping {
                opts: OverlappingOpts::default(),
            },
            tasks: run_items.iter().map(Runnable::from).collect(),
        }
    }
    pub fn as_id(&self, parent: Option<ParentID>) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        parent.hash(&mut hasher);
        hasher.finish()
    }
}

impl TaskSpec {
    pub fn as_tree(&self) -> ArchyNode {
        let path = vec![ParentID::new(self.as_id(None))];
        let label = self.as_tree_label();
        let mut first = ArchyNode::new(&label);
        let empty = HashMap::default();
        append_with_reports(&mut first, &self.tasks, &empty, path);
        first
    }
    pub fn as_tree_with_results(&self, hm: &HashMap<SpecId, TaskReport>) -> ArchyNode {
        let path = vec![ParentID::new(self.as_id(None))];
        let spec_id = SpecId::new(self.as_id(None));
        let r = hm.get(&spec_id);
        let label = match r {
            None => "missing".to_string(),
            Some(_) => self.as_tree_label(),
        };
        let mut first = ArchyNode::new(&label);
        append_with_reports(&mut first, &self.tasks, hm, path);
        first
    }
}

impl TreeDisplay for Runnable {
    fn as_tree_label(&self) -> String {
        match self {
            Runnable::BsLiveTask(item) => format!("{}{}", "Runnable::BsLiveTask", item),
            Runnable::Sh(sh) => format!("{} {}", "Runnable::Sh", sh),
            Runnable::Spec(task_spec) => task_spec.as_tree_label(),
        }
    }
}

pub trait TreeDisplay {
    fn as_tree_label(&self) -> String;
}

impl TaskSpec {
    pub fn to_task_scope(
        self,
        servers_addr: Addr<ServersSupervisor>,
        capabilities_addr: Addr<Capabilities>,
    ) -> TaskScope {
        let parent_id = self.as_id(None);
        let mut tasks = vec![];

        for (_index_position, runnable) in self.tasks.into_iter().enumerate() {
            let item_id = runnable.as_id();

            match runnable {
                Runnable::Spec(task_spec) => {
                    let as_scope =
                        task_spec.to_task_scope(servers_addr.clone(), capabilities_addr.clone());
                    tasks.push(TaskEntry::new(Box::new(as_scope), item_id))
                }
                _ => {
                    let with_ctx = RunnableWithComms {
                        ctx: Comms {
                            servers_addr: servers_addr.clone(),
                            capabilities: capabilities_addr.clone(),
                        },
                        runnable,
                    };
                    tasks.push(TaskEntry::new(Box::new(with_ctx), item_id))
                }
            }
        }

        match self.run_kind {
            RunKind::Sequence { opts } => TaskScope::seq(tasks, opts, parent_id),
            RunKind::Overlapping { opts } => TaskScope::all(tasks, opts, parent_id),
        }
    }
}

pub fn append_with_reports(
    archy: &mut ArchyNode,
    tasks: &[Runnable],
    hm: &HashMap<SpecId, TaskReport>,
    path: Vec<ParentID>,
) {
    for (index_position, runnable) in tasks.iter().enumerate() {
        todo!("now we need to use `iid` to store/lookup reports and overlay in summary");
        let mut next_path = path.clone();
        next_path.push(ParentID::new(index_position as u64));
        // println!("{runnable}");

        let raw_label = runnable.as_tree_label();
        let cid = runnable.as_id();

        next_path.push(ParentID::new(cid));
        // println!(" cid=|| {}", cid.sqid_short());

        {
            let mut iid = DefaultHasher::new();
            cid.hash(&mut iid);
            next_path.hash(&mut iid);
            let iid = iid.finish();
            let path_str = next_path
                .iter()
                .map(|p| p.inner.to_string())
                .collect::<Vec<_>>()
                .join(",");
            // println!(" iid=|| {} || {}", iid.sqid_short(), path_str);
        }

        // {
        //     let mut pid = DefaultHasher::new();
        //     next_path.hash(&mut pid);
        //     let pid = pid.finish();
        //     println!(" pid=|| {}", pid.sqid_short());
        // }

        // dbg!(id);
        match runnable {
            Runnable::BsLiveTask(_) => archy.nodes.push(ArchyNode::new(&raw_label)),
            Runnable::Sh(_) => archy.nodes.push(ArchyNode::new(&raw_label)),
            Runnable::Spec(runner) => {
                let mut next = ArchyNode::new(&raw_label);
                append_with_reports(&mut next, &runner.tasks, hm, next_path);
                archy.nodes.push(next);
            }
        }
    }
}

#[derive(Hash, Debug, Copy, Clone)]
pub struct ParentID {
    inner: u64,
}

impl ParentID {
    pub fn new(id: u64) -> Self {
        Self { inner: id }
    }
}

impl ParentID {
    pub fn id(&self) -> u64 {
        self.inner
    }
}
