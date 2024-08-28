use crate::raw_loader::create_raw_router;
use crate::serve_dir::create_dir_router;
use crate::server::actor::ServerActor;
use actix::ResponseFuture;
use axum::Router;
use bsnext_input::route_manifest::{RouteChangeSet, RoutesManifest};
use bsnext_input::server_config::ServerConfig;
use std::sync::Arc;
use tracing::debug_span;

#[derive(actix::Message, Clone)]
#[rtype(result = "anyhow::Result<RouteChangeSet>")]
pub struct Patch {
    pub server_config: ServerConfig,
}

impl actix::Handler<Patch> for ServerActor {
    type Result = ResponseFuture<anyhow::Result<RouteChangeSet>>;

    fn handle(&mut self, msg: Patch, _ctx: &mut Self::Context) -> Self::Result {
        let span = debug_span!("Patch for ServerActor");
        // todo(alpha): remove this
        let s = Arc::new(span);
        let _g = s.enter();
        // let addr = ctx.address();
        tracing::trace!("Handler<PatchOne> for ServerActor");
        let app_state = self.app_state.as_ref().expect("could not access state");
        let app_state_clone = app_state.clone();
        let routes = msg.server_config.routes.clone();
        let next_manifest = RoutesManifest::new(&routes);
        let changeset = self.routes_manifest.changeset_for(&next_manifest);
        self.routes_manifest = RoutesManifest::new(&routes);

        // let c = s.clone();

        Box::pin(async move {
            let mut mut_raw_router = app_state_clone.raw_router.write().await;
            let mut mut_routes = app_state_clone.routes.write().await;
            *mut_raw_router =
                create_raw_router(&routes).fallback_service(create_dir_router(&routes));
            *mut_routes = routes;

            Ok(changeset)
        })
    }
}
