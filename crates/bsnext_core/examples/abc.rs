use actix::Actor;
use bsnext_core::runtime_ctx::RuntimeCtx;
use bsnext_core::server::actor::ServerActor;
use bsnext_core::server::handler_listen::Listen;
use bsnext_core::servers_supervisor::get_servers_handler::{GetActiveServers, IncomingEvents};
use bsnext_dto::GetActiveServersResponse;
use bsnext_input::route::{JsonWrapper, Route, RouteKind};
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use serde_json::Value;
use tokio::sync::oneshot;

#[actix_rt::main]
async fn main() {
    let (_tx, rx) = oneshot::channel::<()>();
    let route1 = Route {
        path: "/".parse().unwrap(),
        kind: RouteKind::new_html("hello world!"),
        ..Default::default()
    };
    let value: Value = serde_json::from_str("[]").expect("json");
    let route2 = Route {
        path: "/j".parse().unwrap(),
        kind: RouteKind::new_json(JsonWrapper(value)),
        ..Default::default()
    };
    let p = ServerParent::from_routes(vec![route1, route2]);
    let s = ServerActor::new_from_config(p.server_config.clone()).start();
    let parent = p.start();

    let a = s
        .send(Listen {
            runtime_ctx: RuntimeCtx::default(),
            parent: parent.clone().recipient(),
            evt_receiver: parent.recipient(),
        })
        .await;

    match a {
        Ok(Ok(addr)) => {
            dbg!(addr);
            match rx.await {
                Ok(_) => {
                    tracing::info!("servers ended");
                }
                Err(e) => {
                    // dropped? this is ok
                    tracing::trace!(?e, "");
                }
            };
        }
        Ok(Err(e)) => {
            unreachable!("{:?}", e)
        }
        Err(e) => unreachable!("{:?}", e),
    }
}

struct ServerParent {
    server_config: ServerConfig,
}

impl ServerParent {
    pub fn from_routes(routes: Vec<Route>) -> Self {
        let server_config = ServerConfig {
            identity: ServerIdentity::named(),
            routes,
            ..Default::default()
        };

        Self { server_config }
    }
}

impl actix::Actor for ServerParent {
    type Context = actix::Context<Self>;
}
impl actix::Handler<GetActiveServers> for ServerParent {
    type Result = GetActiveServersResponse;

    fn handle(&mut self, _msg: GetActiveServers, _ctx: &mut Self::Context) -> Self::Result {
        todo!("woop!")
    }
}
impl actix::Handler<IncomingEvents> for ServerParent {
    type Result = ();

    fn handle(&mut self, _msg: IncomingEvents, _ctx: &mut Self::Context) -> Self::Result {
        todo!("woop!")
    }
}
