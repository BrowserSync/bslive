use crate::tasks::notify_servers::NotifyServers;
use actix::{
    Actor, ActorFutureExt, Addr, Handler, Recipient, ResponseActFuture, ResponseFuture, Running,
    WrapFuture,
};
use bsnext_core::servers_supervisor::actor::ServersSupervisor;
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal::AnyEvent;
use bsnext_fs::FsEventContext;
use bsnext_input::route::{BsLiveRunner, Runner};
use futures_util::FutureExt;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;

#[derive(actix::Message, Debug, Clone)]
#[rtype(result = "()")]
pub enum TaskCommand {
    Changes {
        changes: Vec<PathBuf>,
        fs_event_context: FsEventContext,
        task_comms: TaskComms,
    },
}

impl TaskCommand {
    pub fn comms(&self) -> &TaskComms {
        match self {
            TaskCommand::Changes { task_comms, .. } => task_comms,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TaskComms {
    pub any_event_sender: Sender<AnyEvent>,
    pub servers_addr: Addr<ServersSupervisor>,
}

impl TaskComms {
    pub(crate) fn new(p0: Sender<AnyEvent>, p1: Addr<ServersSupervisor>) -> TaskComms {
        Self {
            any_event_sender: p0,
            servers_addr: p1,
        }
    }
}

#[derive(Debug)]
pub enum Task {
    Runner(Runner),
    AnyEvent,
    Group(TaskGroup),
}

impl Task {
    pub fn into_actor(self) -> Recipient<TaskCommand> {
        match self {
            Task::Runner(Runner::BsLive {
                bslive: BsLiveRunner::NotifyServer,
            }) => {
                let s = NotifyServers::new();
                let s = s.start();
                s.recipient()
            }
            Task::AnyEvent => {
                let a = AnyEventSender::new();
                let a = a.start();
                a.recipient()
            }
            Task::Group(group) => {
                let group_runner = TaskGroupRunner::new(group);
                let s = group_runner.start();
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

pub struct TaskGroupRunner {
    done: bool,
    task_group: Option<TaskGroup>,
}

impl TaskGroupRunner {
    fn new(task_group: TaskGroup) -> Self {
        Self {
            done: false,
            task_group: Some(task_group),
        }
    }
}

impl actix::Actor for TaskGroupRunner {
    type Context = actix::Context<Self>;

    fn stopped(&mut self, ctx: &mut Self::Context) {
        tracing::trace!(" x stopped TaskGroupRunner")
    }

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        tracing::trace!(" ⏰ stopping TaskGroupRunner {}", self.done);
        Running::Stop
    }
}

impl Handler<TaskCommand> for TaskGroupRunner {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: TaskCommand, ctx: &mut Self::Context) -> Self::Result {
        tracing::debug!(done = self.done, "TaskGroupRunner::TaskCommand");
        let Some(group) = self.task_group.take() else {
            todo!("how to handle a concurrent request here?");
        };
        let future = async move {
            for x in group.tasks {
                let a = x.into_actor();
                match a.send(msg.clone()).await {
                    Ok(_) => tracing::trace!("did send"),
                    Err(e) => tracing::error!("{e}"),
                }
            }
        };
        // let self_addr = ctx.address();
        Box::pin(future.into_actor(self).map(|res, actor, ctx| {
            actor.done = true;
        }))
    }
}

struct AnyEventSender {}

impl AnyEventSender {
    fn new() -> Self {
        Self {}
    }
}

impl actix::Actor for AnyEventSender {
    type Context = actix::Context<Self>;

    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        tracing::trace!(" ⏰ stopping AnyEventSender");
        Running::Stop
    }

    fn stopped(&mut self, ctx: &mut Self::Context) {
        tracing::trace!(" x stopped AnyEventSender");
    }
}
impl Handler<TaskCommand> for AnyEventSender {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: TaskCommand, _ctx: &mut Self::Context) -> Self::Result {
        let evt = match &msg {
            TaskCommand::Changes { changes, .. } => {
                let as_strings = changes
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<String>>();

                AnyEvent::External(ExternalEventsDTO::FilesChanged(
                    bsnext_dto::FilesChangedDTO {
                        paths: as_strings.clone(),
                    },
                ))
            }
        };
        let comms = msg.comms();
        let sender = comms.any_event_sender.clone();
        Box::pin(async move {
            match sender.send(evt).await {
                Ok(_) => tracing::trace!("sent"),
                Err(e) => tracing::error!("{e}"),
            }
        })
    }
}
