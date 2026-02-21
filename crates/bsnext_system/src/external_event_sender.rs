use crate::invoke_scope::{RequestEventSender, TaggedEvent};
use actix::{Handler, Recipient, ResponseFuture, Running};
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal::AnyEvent;
use bsnext_task::invocation::Invocation;
use bsnext_task::task_report::{InvocationId, TaskResult};
use bsnext_task::task_trigger::TaskTriggerSource;

pub struct ExternalEventSenderWithLogging {
    pub request: Recipient<RequestEventSender>,
}

impl ExternalEventSenderWithLogging {
    pub fn new(request: Recipient<RequestEventSender>) -> Self {
        Self { request }
    }
}

impl actix::Actor for ExternalEventSenderWithLogging {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(" started AnyEventSender");
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        tracing::trace!(" ⏰ stopping AnyEventSender");
        Running::Stop
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(" x stopped AnyEventSender");
    }
}

impl Handler<Invocation> for ExternalEventSenderWithLogging {
    type Result = ResponseFuture<TaskResult>;

    fn handle(&mut self, invocation: Invocation, _ctx: &mut Self::Context) -> Self::Result {
        let id = invocation.id;
        let addr = self.request.clone();
        let events: Vec<AnyEvent> = match invocation.trigger.variant {
            TaskTriggerSource::FsChanges { changes, .. } => {
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
            TaskTriggerSource::Exec => vec![],
        };
        Box::pin(async move {
            let Ok(output) = addr.send(RequestEventSender { id }).await else {
                todo!("can this actually fail?");
            };
            for evt in events {
                let tagged = TaggedEvent::new(id, evt);
                match output.sender.send(tagged).await {
                    Ok(_) => tracing::trace!("sent"),
                    Err(e) => tracing::error!("{e}"),
                };
            }
            TaskResult::ok(InvocationId(0))
        })
    }
}
