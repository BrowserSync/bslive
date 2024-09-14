use crate::handler_stack::RouteMap;
use crate::server::actor::ServerActor;
use actix::ResponseFuture;
use bsnext_input::client_config::ClientConfigChangeSet;
use bsnext_input::route_manifest::{RouteChangeSet, RoutesManifest};
use bsnext_input::server_config::ServerConfig;
use tracing::{debug_span, Instrument};

#[derive(actix::Message, Clone)]
#[rtype(result = "anyhow::Result<(RouteChangeSet, ClientConfigChangeSet)>")]
pub struct Patch {
    pub server_config: ServerConfig,
}

impl actix::Handler<Patch> for ServerActor {
    type Result = ResponseFuture<anyhow::Result<(RouteChangeSet, ClientConfigChangeSet)>>;

    fn handle(&mut self, msg: Patch, _ctx: &mut Self::Context) -> Self::Result {
        let span = debug_span!("Patch for ServerActor");

        // todo(alpha): remove this
        let _g = span.enter();

        // Log handler action initiation
        tracing::trace!("Handler<PatchOne> for ServerActor");

        // Access and clone application state
        let app_state = self.app_state.as_ref().expect("could not access state");
        let app_state_clone = app_state.clone();

        // Process routes and manifests
        let routes = msg.server_config.routes.clone();
        let next_manifest = RoutesManifest::new(&routes);
        let changeset = self.routes_manifest.changeset_for(&next_manifest);
        self.routes_manifest = RoutesManifest::new(&routes);

        // Process client configuration changes
        let client_config = msg.server_config.clients.clone();
        let client_config_change_set = self
            .config
            .clients
            .changeset_for(&msg.server_config.clients);

        // todo(alpha): use the actix dedicated methods for async state mutation?
        Box::pin({
            let c = span.clone();

            async move {
                let router = RouteMap::new_from_routes(&routes).into_router();
                let mut mut_raw_router = app_state_clone.raw_router.write().await;
                *mut_raw_router = router;
                drop(mut_raw_router);
                tracing::trace!("did update raw_router");

                let mut mut_routes = app_state_clone.routes.write().await;
                *mut_routes = routes;
                drop(mut_routes);
                tracing::trace!("did update routes");

                let mut mut_client_config = app_state_clone.client_config.write().await;
                *mut_client_config = client_config;
                drop(mut_client_config);
                tracing::trace!("did update client_config");

                Ok((changeset, client_config_change_set))
            }
            .instrument(c)
        })
    }
}
