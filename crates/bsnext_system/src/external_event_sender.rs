use crate::capabilities::output_channel::RequestOutputChannel;
use crate::capabilities::TaggedEvent;
use actix::{Handler, Recipient, ResponseFuture, Running};
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal::AnyEvent;
use bsnext_task::invocation::Invocation;
use bsnext_task::invocation::InvocationId;
use bsnext_task::invocation_result::InvocationResult;
use bsnext_task::task_trigger::TaskTriggerSource;

pub struct ExternalEventSenderWithLogging {
    pub request: Recipient<RequestOutputChannel>,
}

impl ExternalEventSenderWithLogging {
    pub fn new(request: Recipient<RequestOutputChannel>) -> Self {
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
    type Result = ResponseFuture<InvocationResult>;

    fn handle(&mut self, invocation: Invocation, _ctx: &mut Self::Context) -> Self::Result {
        let id = invocation.id;
        let addr = self.request.clone();
        let events: Vec<AnyEvent> = match invocation.trigger.trigger_source {
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
            let Ok(Ok(output)) = addr.send(RequestOutputChannel { invocation_id: id }).await else {
                todo!("can this actually fail?");
            };
            for evt in events {
                let tagged = TaggedEvent::new(id.u64(), evt);
                match output.sender.send(tagged).await {
                    Ok(_) => tracing::trace!("sent"),
                    Err(e) => tracing::error!("{e}"),
                };
            }
            InvocationResult::ok(InvocationId::new(0))
        })
    }
}
