use crate::server::actor::ServerActor;
use bsnext_dto::{ClientEvent, DisplayMessageDTO};
use bsnext_input::bs_live_built_in_task::ClientNotification;

#[derive(actix::Message, Debug)]
#[rtype(result = "()")]
pub enum Notification {
    Any(ClientNotification),
}

impl actix::Handler<Notification> for ServerActor {
    type Result = ();

    fn handle(&mut self, msg: Notification, _ctx: &mut Self::Context) -> Self::Result {
        let Some(client_sender) = self.client_sender() else {
            return tracing::error!("signals not ready, should they be?");
        };

        match msg {
            Notification::Any(ClientNotification::DisplayMessage(dm)) => {
                let msg = DisplayMessageDTO {
                    message: dm.message,
                    reason: dm.reason,
                };
                match client_sender.send(ClientEvent::DisplayMessage(msg)) {
                    Ok(_) => tracing::debug!("send ClientEvent::DisplayMessage to clients"),
                    Err(e) => {
                        tracing::error!("did not send ClientEvent::DisplayMessage to clients");
                        tracing::error!(?e)
                    }
                }
            }
        }
    }
}
