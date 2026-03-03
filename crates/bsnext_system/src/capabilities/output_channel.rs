use crate::capabilities::{Capabilities, TaggedEvent};
use actix::{Handler, ResponseFuture};
use actix_rt::Arbiter;
use bsnext_task::invocation::InvocationId;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

pub struct OutputChannel {
    pub sender: tokio::sync::mpsc::Sender<TaggedEvent>,
}

#[derive(actix::Message)]
#[rtype(result = "Result<OutputChannel, anyhow::Error>")]
pub struct RequestOutputChannel {
    pub invocation_id: InvocationId,
}

impl RequestOutputChannel {
    pub fn sqid(&self) -> String {
        self.invocation_id.sqid()
    }
}

impl Handler<RequestOutputChannel> for Capabilities {
    type Result = ResponseFuture<Result<OutputChannel, anyhow::Error>>;

    #[tracing::instrument(skip_all, fields(invocation_id = msg.invocation_id.u64(), sqid = msg.sqid()), name = "RequestOutputChannel")]
    fn handle(&mut self, msg: RequestOutputChannel, _ctx: &mut Self::Context) -> Self::Result {
        let (tx, rx) = tokio::sync::mpsc::channel::<TaggedEvent>(100);
        // keeping this as a stream for future things like combinators
        let mut stream = ReceiverStream::new(rx);
        let id = msg.invocation_id;
        Arbiter::current().spawn({
            let events_sender = self.any_event_sender.clone();
            async move {
                while let Some(evt) = stream.next().await {
                    tracing::trace!(output.id = id.u64(), output.evt = ?evt.event);
                    match events_sender.send(evt.event).await {
                        Ok(_) => {}
                        Err(_) => tracing::error!("could not send"),
                    }
                }
            }
        });
        Box::pin(async move { Ok(OutputChannel { sender: tx }) })
    }
}
