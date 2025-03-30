use crate::task::TaskCommand;
use actix::{Handler, ResponseFuture, Running};
use bsnext_dto::internal::AnyEvent;

pub struct AnyEventSender {
    evt: Option<AnyEvent>,
}

impl AnyEventSender {
    pub fn new(evt: AnyEvent) -> Self {
        Self { evt: Some(evt) }
    }
}

impl actix::Actor for AnyEventSender {
    type Context = actix::Context<Self>;

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        tracing::trace!(" ⏰ stopping AnyEventSender");
        Running::Stop
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(" x stopped AnyEventSender");
        println!(" x stopped AnyEventSender");
    }
    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(" started AnyEventSender");
        println!(" started AnyEventSender");
    }
}

impl Handler<TaskCommand> for AnyEventSender {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: TaskCommand, _ctx: &mut Self::Context) -> Self::Result {
        let evt = self.evt.take().expect("must have an event here");
        let comms = msg.comms();
        let sender = comms.any_event_sender.clone();
        Box::pin(async move {
            match sender.send(evt).await {
                Ok(_) => tracing::trace!("sent"),
                Err(e) => tracing::error!("{e}"),
            }
            println!("done 1");
        })
    }
}
