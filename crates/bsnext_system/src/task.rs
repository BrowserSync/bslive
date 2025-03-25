use crate::tasks::notify_servers::NotifyServers;
use crate::BsSystem;
use actix::{Actor, Addr, Recipient};
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_dto::internal::AnyEvent;
use bsnext_fs::FsEventContext;
use bsnext_input::route::BsLiveRunner::NotifyServer;
use bsnext_input::route::{BsLiveRunner, Runner};
use std::path::PathBuf;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
pub enum TaskCommand {
    Changes {
        changes: Vec<PathBuf>,
        fs_event_context: FsEventContext,
    },
}

#[derive(Debug)]
pub struct TaskManager {
    running: bool,
    addr: Recipient<TaskCommand>,
}

impl TaskManager {
    pub fn new(addr: Recipient<TaskCommand>) -> Self {
        TaskManager {
            addr,
            running: false,
        }
    }

    pub fn addr(&self) -> &Recipient<TaskCommand> {
        &self.addr
    }
}

#[derive(Debug)]
pub enum Task {
    Runner(Runner),
    AnyEvent(AnyEvent),
    Group(TaskGroup),
}

impl Task {
    pub fn into_actor(self, servers_addr: Addr<ServersSupervisor>) -> Recipient<TaskCommand> {
        match self {
            Task::Runner(Runner::BsLive {
                bslive: BsLiveRunner::NotifyServer,
            }) => {
                let servers_addr = servers_addr.clone();
                let s = NotifyServers::new(servers_addr);
                let s = s.start();
                s.recipient()
            }
            Task::AnyEvent(_) => todo!("Task::AnyEvent"),
            Task::Group(group) => {
                let servers_addr = servers_addr.clone();
                let s = NotifyServers::new(servers_addr);
                let s = s.start();
                s.recipient()
            }
            Task::Runner(Runner::Sh { .. }) => todo!("Task::Runner::Runner::Sh"),
            Task::Runner(Runner::ShImplicit(_)) => todo!("Task::Runner::Runner::ShImplicit"),
        }
    }
}

#[derive(Debug)]
pub struct TaskGroup {
    tasks: Vec<Task>,
}

impl TaskGroup {
    pub fn new(tasks: Vec<Task>) -> Self {
        Self { tasks }
    }

    pub fn len(&self) -> usize {
        self.tasks.len()
    }
}
