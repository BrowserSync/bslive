use crate::server::actor::ServerActor;
use bsnext_dto::ClientEvent;
use bsnext_input::client_config::ClientConfigChangeSet;
use tracing::trace_span;

#[derive(actix::Message, Clone, Debug)]
#[rtype(result = "()")]
pub struct ClientConfigChange {
    pub change_set: ClientConfigChangeSet,
}

impl actix::Handler<ClientConfigChange> for ServerActor {
    type Result = ();

    fn handle(&mut self, msg: ClientConfigChange, _ctx: &mut Self::Context) -> Self::Result {
        let s = trace_span!("ClientConfigChange for ServerActor");
        let _g = s.enter();

        let sender = self.signals.as_ref().and_then(|s| s.client_sender.as_ref());
        let first_change = msg.change_set.changed.first();

        match (sender, first_change) {
            (Some(client_sender), Some(changed_config)) => {
                tracing::info!("forwarding `ClientConfigChange` event to web socket clients");

                match client_sender.send(ClientEvent::Config(changed_config.into())) {
                    Ok(_) => tracing::trace!("ClientConfigChange event sent to clients"),
                    Err(_) => tracing::error!("ClientConfigChange not sent to client_sender"),
                };
            }
            (Some(..), None) => {
                tracing::trace!("no config changed, bailing");
            }
            _ => {
                tracing::debug!("could not process ClientConfigChange properly");
            }
        }
    }
}
