use crate::capabilities::{Capabilities, TaggedEvent};
use actix::{Handler, ResponseFuture};
use actix_rt::Arbiter;
use bsnext_task::invocation::SpecId;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

pub struct OutputChannel {
    pub sender: tokio::sync::mpsc::Sender<TaggedEvent>,
}

#[derive(actix::Message)]
#[rtype(result = "Result<OutputChannel, anyhow::Error>")]
pub struct RequestOutputChannel {
    pub spec_id: SpecId,
}

impl RequestOutputChannel {
    pub fn sqid(&self) -> String {
        self.spec_id.sqid()
    }
}

impl Handler<RequestOutputChannel> for Capabilities {
    type Result = ResponseFuture<Result<OutputChannel, anyhow::Error>>;

    #[tracing::instrument(skip_all, fields(spec_id = msg.spec_id.u64(), sqid = msg.sqid()), name = "RequestOutputChannel")]
    fn handle(&mut self, msg: RequestOutputChannel, _ctx: &mut Self::Context) -> Self::Result {
        let (tx, rx) = tokio::sync::mpsc::channel::<TaggedEvent>(100);
        // keeping this as a stream for future things like combinators
        let spec_id = msg.spec_id;
        Arbiter::current().spawn({
            let mut stream = ReceiverStream::new(rx);
            let events_sender = self.any_event_sender.clone();
            async move {
                while let Some(evt) = stream.next().await {
                    tracing::trace!(output.id = spec_id.u64(), output.evt = ?evt.event);
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
