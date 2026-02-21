use crate::external_event_sender::ExternalEventSenderWithLogging;
use crate::tasks::notify_servers::{NotifyServers, NotifyServersNoOp};
use crate::tasks::sh_cmd::ShCmd;
use crate::tasks::task_spec::{TaskSpec, TreeDisplay};
use crate::BsSystem;
use actix::{Actor, Addr, Recipient};
use bs_live_task::BsLiveTask;
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_dto::archy::ArchyNode;
use bsnext_input::route::{BsLiveRunner, RunAll, RunOptItem, RunSeq};
use bsnext_task::as_actor::AsActor;
use bsnext_task::invocation::Invocation;
use bsnext_task::task_report::TaskReport;
use bsnext_task::{OverlappingOpts, SequenceOpts};
use std::collections::HashMap;
use std::hash::{DefaultHasher, Hash, Hasher};

pub mod bs_live_task;
pub mod notify_servers;
pub mod sh_cmd;
pub mod task_comms;
pub mod task_spec;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum Runnable {
    BsLiveTask(BsLiveTask),
    Sh(ShCmd),
    Many(TaskSpec),
}

#[derive(Debug)]
pub struct RunnableWithComms {
    runnable: Runnable,
    ctx: Comms,
}

#[derive(Debug, Clone)]
pub struct Comms {
    servers_addr: Option<Addr<ServersSupervisor>>,
    sys: Addr<BsSystem>,
}

pub trait IntoRecipient {
    fn into_recipient(self: Self, addr: &Addr<BsSystem>) -> Recipient<Invocation>;
}

impl AsActor for RunnableWithComms {
    fn into_task_recipient(self: Box<Self>) -> Recipient<Invocation> {
        match self.runnable {
            Runnable::BsLiveTask(BsLiveTask::NotifyServer) => match self.ctx.servers_addr {
                None => {
                    let s = NotifyServersNoOp;
                    let s = s.start();
                    s.recipient()
                }
                Some(addr) => {
                    let s = NotifyServers::new(addr.clone());
                    let s = s.start();
                    s.recipient()
                }
            },
            Runnable::BsLiveTask(BsLiveTask::PublishExternalEvent) => {
                let actor = ExternalEventSenderWithLogging::new(self.ctx.sys.recipient());
                let addr = actor.start();
                addr.recipient()
            }
            Runnable::Sh(sh) => sh.into_recipient(&self.ctx.sys),
            Runnable::Many(_) => unreachable!("The conversion to Task happens elsewhere"),
        }
    }
}

fn append_with_reports(archy: &mut ArchyNode, tasks: &[Runnable], hm: &HashMap<u64, TaskReport>) {
    for (index_position, runnable) in tasks.iter().enumerate() {
        let id = runnable.as_id_with(Index(index_position as u64));
        let sqid = runnable.as_sqid(id);
        let label_with_id = match hm.get(&id) {
            None => format!(
                "[{sqid}] − {}",
                runnable.as_tree_label(Index(index_position as u64))
            ),
            Some(report) => {
                if runnable.is_group() {
                    runnable.as_tree_label(Index(index_position as u64))
                } else {
                    format!(
                        "[{sqid}] {} {}",
                        if report.is_ok() { "✅" } else { "❌" },
                        runnable.as_tree_label(Index(index_position as u64))
                    )
                }
            }
        };
        let raw_label = match hm.get(&id) {
            None => format!("{}", runnable.as_tree_label(Index(index_position as u64))),
            Some(report) => {
                if runnable.is_group() {
                    runnable.as_tree_label(Index(index_position as u64))
                } else {
                    format!(
                        "{} {}",
                        if report.is_ok() { "✅" } else { "❌" },
                        runnable.as_tree_label(Index(index_position as u64))
                    )
                }
            }
        };
        match runnable {
            Runnable::BsLiveTask(_) => archy.nodes.push(ArchyNode::new(&label_with_id)),
            Runnable::Sh(_) => archy.nodes.push(ArchyNode::new(&label_with_id)),
            Runnable::Many(runner) => {
                let mut next = ArchyNode::new(&raw_label);
                append_with_reports(&mut next, &runner.tasks, hm);
                archy.nodes.push(next);
            }
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
    pub fn as_id_with(&self, Index(index): Index) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        index.hash(&mut hasher);
        hasher.finish()
    }
    pub fn as_sqid(&self, id: u64) -> String {
        let sqids = sqids::Sqids::default();
        let sqid = sqids.encode(&[id]).unwrap_or_else(|_| id.to_string());
        sqid.get(0..6).map(String::from).unwrap_or(sqid)
    }
}

pub struct Index(pub u64);

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
                Self::Many(TaskSpec::all(&items, opts))
            }
            RunOptItem::Seq(RunSeq { seq, seq_opts }) => {
                let items: Vec<_> = seq.iter().map(Runnable::from).collect();
                let opts = SequenceOpts {
                    exit_on_failure: seq_opts.exit_on_fail,
                };
                Self::Many(TaskSpec::seq_opts(&items, opts))
            }
        }
    }
}

impl From<RunOptItem> for Runnable {
    fn from(value: RunOptItem) -> Self {
        Runnable::from(&value)
    }
}
