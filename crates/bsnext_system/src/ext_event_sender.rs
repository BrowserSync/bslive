use crate::task::TaskCommand;
use actix::{Handler, ResponseFuture, Running};
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal::{AnyEvent, InvocationId, TaskResult};

#[derive(Default, Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct ExtEventSender;

impl ExtEventSender {
    pub fn new() -> Self {
        Self {}
    }
}

impl actix::Actor for ExtEventSender {
    type Context = actix::Context<Self>;

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        tracing::trace!(" ⏰ stopping AnyEventSender");
        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(" x stopped AnyEventSender");
    }
    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(" started AnyEventSender");
    }
}

impl Handler<TaskCommand> for ExtEventSender {
    type Result = ResponseFuture<TaskResult>;

    fn handle(&mut self, msg: TaskCommand, _ctx: &mut Self::Context) -> Self::Result {
        let comms = msg.comms();
        let sender = comms.any_event_sender.clone();
        let events: Vec<AnyEvent> = match msg {
            TaskCommand::Changes { changes, .. } => {
                let as_strings = changes
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<String>>();

                vec![AnyEvent::External(ExternalEventsDTO::FilesChanged(
                    bsnext_dto::FilesChangedDTO {
                        paths: as_strings.clone(),
                    },
                ))]
            }
        };
        Box::pin(async move {
            for evt in events {
                match sender.send(evt).await {
                    Ok(_) => tracing::trace!("sent"),
                    Err(e) => tracing::error!("{e}"),
                };
            }
            TaskResult::ok(InvocationId(0))
        })
    }
}
