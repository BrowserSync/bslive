use crate::capabilities::Capabilities;
use crate::tasks::comms::Comms;
use crate::tasks::{Node, Runnable, RunnableWithComms, Sqid};
use actix::Addr;
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_dto::archy::ArchyNode;
use bsnext_input::route::RunOptItem;
use bsnext_task::task_entry::TaskEntry;
use bsnext_task::task_report::TaskReport;
use bsnext_task::task_scope::TaskScope;
use bsnext_task::{
    ContentId, IndexId, NodePath, OverlappingOpts, PathSegment, RunKind, SequenceOpts,
};
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
    tasks: Vec<Node>,
    path: NodePath,
}

impl TaskSpec {
    pub fn path(&self) -> &NodePath {
        &self.path
    }
    pub fn is_seq(&self) -> bool {
        matches!(self.run_kind, RunKind::Sequence { .. })
    }
    pub fn is_all(&self) -> bool {
        matches!(self.run_kind, RunKind::Overlapping { .. })
    }
    pub fn all(tasks: &[Runnable], opts: OverlappingOpts) -> Self {
        todo!("lo");
        let nodes = tasks
            .into_iter()
            .map(|r| Node {
                node: r.clone(),
                path: Default::default(),
            })
            .collect();
        Self {
            run_kind: RunKind::Overlapping { opts },
            tasks: nodes,
            path: Default::default(),
        }
    }
    pub fn seq(tasks: &[Runnable]) -> Self {
        todo!("lo");
        let nodes = tasks
            .into_iter()
            .map(|r| Node {
                node: r.clone(),
                path: Default::default(),
            })
            .collect();
        Self {
            run_kind: RunKind::Sequence {
                opts: SequenceOpts::default(),
            },
            tasks: nodes,
            path: Default::default(),
        }
    }
    pub fn seq_opts(tasks: &[Runnable], opts: SequenceOpts) -> Self {
        let nodes = tasks
            .into_iter()
            .map(|r| Node {
                node: r.clone(),
                path: Default::default(),
            })
            .collect();
        let mut item = Self {
            run_kind: RunKind::Sequence { opts },
            tasks: nodes,
            path: Default::default(),
        };
        let p = NodePath::root_for(ContentId::new(item.as_id()));
        item.annotate(p);
        item
    }
    pub fn seq_from(run_items: &[RunOptItem]) -> Self {
        // todo!("lo");
        let nodes = run_items
            .into_iter()
            .map(Runnable::from)
            .map(|runnable| Node {
                node: runnable,
                path: Default::default(),
            })
            .collect();
        let mut item = Self {
            run_kind: RunKind::Sequence {
                opts: SequenceOpts::default(),
            },
            tasks: nodes,
            path: Default::default(),
        };
        let p = NodePath::root_for(ContentId::new(item.as_id()));
        item.annotate(p);
        item
    }
    pub fn all_from(run_items: &[RunOptItem]) -> Self {
        todo!("lo");
        let nodes = run_items
            .into_iter()
            .map(Runnable::from)
            .map(|runnable| Node {
                node: runnable,
                path: Default::default(),
            })
            .collect();
        Self {
            run_kind: RunKind::Overlapping {
                opts: OverlappingOpts::default(),
            },
            tasks: nodes,
            path: Default::default(),
        }
    }
    pub fn as_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    fn annotate(&mut self, mut path: NodePath) {
        let mut index = 0;
        self.path = path.clone();

        for runnable in &mut self.tasks {
            let mut next_path = path.clone();
            next_path.append(PathSegment::Index(IndexId::new(index as u64)));

            match runnable.node {
                Runnable::BsLiveTask(_) => {
                    next_path.append(PathSegment::Content(runnable.content_id()));
                    runnable.path = next_path;
                }
                Runnable::Sh(_) => {
                    next_path.append(PathSegment::Content(runnable.content_id()));
                    runnable.path = next_path;
                }
                Runnable::Spec(ref mut spec) => {
                    next_path.append(PathSegment::Content(ContentId::new(spec.as_id())));
                    runnable.path = next_path.clone();
                    spec.annotate(next_path);
                }
            }

            index = index + 1;
        }
    }
}

impl TaskSpec {
    pub fn as_tree(&self) -> ArchyNode {
        let empty = HashMap::default();
        let label = self.as_tree_label();
        let mut first = ArchyNode::new(&label);
        append_with_reports(&mut first, &self.tasks, &empty);
        first
    }
    pub fn as_tree_with_results(&self, hm: &HashMap<NodePath, TaskReport>) -> ArchyNode {
        let node_path = self.path().to_owned();
        let r = hm.get(&node_path);
        let label = match r {
            None => "missing".to_string(),
            Some(_) => self.as_tree_label(),
        };
        let mut first = ArchyNode::new(&label);
        append_with_reports(&mut first, &self.tasks, hm);
        first
    }
}

pub trait TreeDisplay {
    fn as_tree_label(&self) -> String;
}

impl TreeDisplay for TaskSpec {
    fn as_tree_label(&self) -> String {
        let p = &self.path;
        format!("{p}")
    }
}

impl TaskSpec {
    pub fn to_task_scope(
        self,
        servers_addr: Addr<ServersSupervisor>,
        capabilities_addr: Addr<Capabilities>,
    ) -> TaskScope {
        let parent_id = self.as_id();
        let mut tasks = vec![];

        for (_index_position, runnable) in self.tasks.into_iter().enumerate() {
            let item_id = runnable.content_id();

            match runnable.node {
                Runnable::Spec(task_spec) => {
                    let path = task_spec.path().to_owned();
                    let as_scope =
                        task_spec.to_task_scope(servers_addr.clone(), capabilities_addr.clone());
                    tasks.push(TaskEntry::new(Box::new(as_scope), item_id, path))
                }
                _ => {
                    let with_ctx = RunnableWithComms {
                        ctx: Comms {
                            servers_addr: servers_addr.clone(),
                            capabilities: capabilities_addr.clone(),
                        },
                        runnable,
                    };
                    let path = self.path.to_owned();
                    tasks.push(TaskEntry::new(Box::new(with_ctx), item_id, path))
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
    tasks: &[Node],
    hm: &HashMap<NodePath, TaskReport>,
) {
    for (index_position, node) in tasks.iter().enumerate() {
        let raw_label = node.as_tree_label();
        match &node.node {
            Runnable::BsLiveTask(_) => archy.nodes.push(ArchyNode::new(&raw_label)),
            Runnable::Sh(_) => archy.nodes.push(ArchyNode::new(&raw_label)),
            Runnable::Spec(runner) => {
                let mut next = ArchyNode::new(&raw_label);
                append_with_reports(&mut next, &runner.tasks, hm);
                archy.nodes.push(next);
            }
        }
    }
}
