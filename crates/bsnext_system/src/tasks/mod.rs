use crate::external_event_sender::ExternalEventSenderWithLogging;
use crate::tasks::notify_servers::NotifyServersReady;
use crate::tasks::sh_cmd::ShCmd;
use crate::tasks::task_spec::TaskSpec;
use actix::{Actor, Recipient};
use bs_live_task::BsLiveTask;
use bsnext_input::route::{BsLiveRunner, RunAll, RunOptItem, RunSeq};
use bsnext_task::as_actor::AsActor;
use bsnext_task::invocation::Invocation;
use bsnext_task::{OverlappingOpts, SequenceOpts};
use comms::Comms;
use into_recipient::IntoRecipient;
use std::hash::{DefaultHasher, Hash, Hasher};
use task_spec::ParentID;

pub mod bs_live_task;
pub mod comms;
mod into_recipient;
pub mod notify_servers;
pub mod resolve;
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

impl AsActor for RunnableWithComms {
    fn into_task_recipient(self: Box<Self>) -> Recipient<Invocation> {
        match self.runnable {
            Runnable::BsLiveTask(BsLiveTask::NotifyServer) => {
                let a = NotifyServersReady {
                    addr: self.ctx.capabilities.recipient(),
                };
                let actor = a.start();
                actor.recipient()
            }
            Runnable::BsLiveTask(BsLiveTask::PublishExternalEvent) => {
                let actor = ExternalEventSenderWithLogging::new(self.ctx.capabilities.recipient());
                let addr = actor.start();
                addr.recipient()
            }
            Runnable::Sh(sh) => sh.into_recipient(&self.ctx.capabilities),
            Runnable::Many(_) => unreachable!("The conversion to Task happens elsewhere"),
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
    pub fn as_id_with(&self, ParentID(index): ParentID) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        index.hash(&mut hasher);
        hasher.finish()
    }
    pub fn as_id_with_path(&self, path: &[ParentID]) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        path.hash(&mut hasher);
        hasher.finish()
    }
    pub fn as_sqid(&self, id: u64) -> String {
        let sqids = sqids::Sqids::default();
        let sqid = sqids.encode(&[id]).unwrap_or_else(|_| id.to_string());
        sqid.get(0..6).map(String::from).unwrap_or(sqid)
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
