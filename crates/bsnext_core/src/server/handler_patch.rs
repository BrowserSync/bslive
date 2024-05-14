use crate::server::actor::ServerActor;
use crate::server::handler_routes_updated::RoutesUpdated;
use actix::AsyncContext;
use actix_rt::Arbiter;
use anyhow::anyhow;
use bsnext_input::route_manifest::RoutesManifest;
use bsnext_input::server_config::ServerConfig;

#[derive(actix::Message, Clone)]
#[rtype(result = "anyhow::Result<()>")]
pub struct Patch {
    pub server_config: ServerConfig,
}

impl actix::Handler<Patch> for ServerActor {
    type Result = anyhow::Result<()>;

    fn handle(&mut self, msg: Patch, ctx: &mut Self::Context) -> Self::Result {
        let addr = ctx.address();
        tracing::trace!("Handler<PatchOne> for ServerActor");
        let app_state = self
            .app_state
            .as_ref()
            .ok_or(anyhow!("could not access state"))?;
        let app_state_clone = app_state.clone();
        let routes = msg.server_config.routes.clone();
        let next_manifest = RoutesManifest::new(&routes);
        let changeset = self.routes_manifest.changeset_for(&next_manifest);
        self.routes_manifest = RoutesManifest::new(&routes);

        let update_dn = async move {
            let mut mut_routes = app_state_clone.routes.write().await;
            *mut_routes = routes;
            addr.do_send(RoutesUpdated {
                change_set: changeset,
            })
        };

        Arbiter::current().spawn(update_dn);
        Ok(())
    }
}
