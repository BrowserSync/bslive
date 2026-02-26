use crate::capabilities::{Capabilities, TaggedEvent};
use actix::{Handler, ResponseFuture};
use actix_rt::Arbiter;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

pub struct OutputChannel {
    pub sender: tokio::sync::mpsc::Sender<TaggedEvent>,
}

#[derive(actix::Message)]
#[rtype(result = "Result<OutputChannel, anyhow::Error>")]
pub struct RequestOutputChannel {
    pub id: u64,
}

impl Handler<RequestOutputChannel> for Capabilities {
    type Result = ResponseFuture<Result<OutputChannel, anyhow::Error>>;

    fn handle(&mut self, _msg: RequestOutputChannel, ctx: &mut Self::Context) -> Self::Result {
        let (tx, rx) = tokio::sync::mpsc::channel::<TaggedEvent>(100);
        // keeping this as a stream for future things like combinators
        let mut stream = ReceiverStream::new(rx);
        Arbiter::current().spawn({
            let events_sender = self.any_event_sender.clone();
            async move {
                while let Some(evt) = stream.next().await {
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
