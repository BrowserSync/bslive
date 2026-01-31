use crate::external_event_sender::ExternalEventSender;
use crate::tasks::notify_servers::NotifyServers;
use crate::tasks::sh_cmd::ShCmd;
use crate::tasks::task_list::{TaskList, TreeDisplay};
use actix::{Actor, Recipient};
use bs_live_task::BsLiveTask;
use bsnext_dto::archy::ArchyNode;
use bsnext_dto::internal::TaskReport;
use bsnext_task::as_actor::AsActor;
use bsnext_task::invocation::Invocation;
use std::collections::HashMap;

pub mod bs_live_task;
pub mod notify_servers;
pub mod sh_cmd;
pub mod task_list;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum Runnable {
    BsLiveTask(BsLiveTask),
    Sh(ShCmd),
    Many(TaskList),
}

impl AsActor for Runnable {
    fn into_task_recipient(self: Box<Self>) -> Recipient<Invocation> {
        match *self {
            Runnable::BsLiveTask(BsLiveTask::NotifyServer) => {
                let s = NotifyServers::new();
                let s = s.start();
                s.recipient()
            }
            Runnable::BsLiveTask(BsLiveTask::PublishExternalEvent) => {
                let actor = ExternalEventSender::new();
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
        let sqid = runnable.as_sqid(id);
        let label = match hm.get(&id) {
            None => format!("[{sqid}] − {}", runnable.as_tree_label(i as u64)),
            Some(report) => {
                if runnable.is_group() {
                    runnable.as_tree_label(i as u64)
                } else {
                    format!(
                        "[{sqid}] {} {}",
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
