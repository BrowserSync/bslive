use crate::external_event_sender::ExternalEventSenderWithLogging;
use crate::tasks::notify_servers::NotifyServersReady;
use crate::tasks::sh_cmd::ShCmd;
use crate::tasks::task_spec::{TaskSpec, TreeDisplay};
use actix::{Actor, Recipient};
use bs_live_task::BsLiveTask;
use bsnext_input::route::{BsLiveRunner, RunAll, RunOptItem, RunSeq};
use bsnext_task::as_actor::AsActor;
use bsnext_task::invocation::Invocation;
use bsnext_task::{ContentId, NodePath, OverlappingOpts, PathSegment, SequenceOpts};
use comms::Comms;
use into_recipient::IntoRecipient;
use std::fmt::{Display, Formatter};
use std::hash::{DefaultHasher, Hash, Hasher};

trait Sqid {
    fn sqid(&self) -> String;
    fn sqid_short(&self) -> String;
}

impl Sqid for u64 {
    fn sqid(&self) -> String {
        let sqids = sqids::Sqids::default();
        sqids.encode(&[*self]).unwrap_or_else(|_| self.to_string())
    }

    fn sqid_short(&self) -> String {
        let sqids = sqids::Sqids::default();
        let sqid = sqids.encode(&[*self]).unwrap();
        sqid.get(0..6).unwrap_or(&sqid).to_string()
    }
}

pub mod bs_live_task;
pub mod comms;
mod into_recipient;
pub mod notify_servers;
pub mod resolve;
pub mod sh_cmd;
pub mod task_comms;
pub mod task_spec;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct Node {
    node: Runnable,
    path: NodePath,
}

impl Node {
    pub fn content_id(&self) -> ContentId {
        self.node.content_id()
    }
    pub fn path(&self) -> &NodePath {
        &self.path
    }
}

impl TreeDisplay for Node {
    fn as_tree_label(&self) -> String {
        let p = &self.path;
        let p2 = self.path.path_hash().sqid_short();
        todo!("where to use path hash over raw path?");
        todo!("Also, implement path for node only, not separately for node vs task spec");
        let p = format!("[{p}]");
        match &self.node {
            Runnable::BsLiveTask(item) => format!("{p} {}{}", "Runnable::BsLiveTask", item),
            Runnable::Sh(sh) => format!("{p} {} {}", "Runnable::Sh", sh),
            Runnable::Spec(task_spec) => task_spec.as_tree_label(),
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum Runnable {
    BsLiveTask(BsLiveTask),
    Sh(ShCmd),
    Spec(TaskSpec),
}

impl Display for Runnable {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Runnable::BsLiveTask(_) => write!(f, "{}", "Runnable::BsLiveTask"),
            Runnable::Sh(_) => write!(f, "{}", "Runnable::Sh"),
            Runnable::Spec(spec) if spec.is_seq() => {
                write!(f, "{}", "Runnable::Spec (seq)")
            }
            Runnable::Spec(spec) => {
                write!(f, "{}", "Runnable::Spec (all)")
            }
        }
    }
}

#[derive(Debug)]
pub struct RunnableWithComms {
    runnable: Node,
    ctx: Comms,
}

impl AsActor for RunnableWithComms {
    fn into_task_recipient(self: Box<Self>) -> Recipient<Invocation> {
        match self.runnable.node {
            Runnable::BsLiveTask(BsLiveTask::NotifyServer) => {
                let a = NotifyServersReady::new(self.ctx.capabilities.recipient());
                let actor = a.start();
                actor.recipient()
            }
            Runnable::BsLiveTask(BsLiveTask::PublishExternalEvent) => {
                let actor = ExternalEventSenderWithLogging::new(self.ctx.capabilities.recipient());
                let addr = actor.start();
                addr.recipient()
            }
            Runnable::Sh(sh) => sh.into_recipient(&self.ctx.capabilities),
            Runnable::Spec(_) => unreachable!("The conversion to Task happens elsewhere"),
        }
    }
}

impl Runnable {
    pub fn is_group(&self) -> bool {
        match self {
            Runnable::BsLiveTask(_) => false,
            Runnable::Sh(_) => false,
            Runnable::Spec(_) => true,
        }
    }
    pub fn content_id(&self) -> ContentId {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        ContentId::new(hasher.finish())
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
                Self::Spec(TaskSpec::all(&items, opts))
            }
            RunOptItem::Seq(RunSeq { seq, seq_opts }) => {
                let items: Vec<_> = seq.iter().map(Runnable::from).collect();
                let opts = SequenceOpts {
                    exit_on_failure: seq_opts.exit_on_fail,
                };
                Self::Spec(TaskSpec::seq_opts(&items, opts))
            }
        }
    }
}

impl From<RunOptItem> for Runnable {
    fn from(value: RunOptItem) -> Self {
        Runnable::from(&value)
    }
}
