use actix::Actor;
use bsnext_core::server::actor::ServerActor;
use bsnext_core::server::handler_listen::Listen;
use bsnext_core::servers_supervisor::file_changed_handler::FilesChanged;
use bsnext_core::servers_supervisor::get_servers_handler::{GetServersMessage, IncomingEvents};
use bsnext_dto::GetServersMessageResponse;
use bsnext_input::route::{JsonWrapper, Route, RouteKind};
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use http::header::ACCEPT;
use mime_guess::mime::{APPLICATION_JSON, TEXT_HTML_UTF_8};
use serde_json::Value;
use std::path::PathBuf;

mod inject_tests;
mod tests;

async fn system_test_01() {
    let route1 = Route {
        path: "/".to_string(),
        kind: RouteKind::new_html("hello world!"),
        ..Default::default()
    };
    let value: Value = serde_json::from_str("[]").expect("json");
    let route2 = Route {
        path: "/j".to_string(),
        kind: RouteKind::new_json(JsonWrapper(value)),
        ..Default::default()
    };
    let p = ServerParent::from_routes(vec![route1, route2]);
    let s = ServerActor::new_from_config(p.server_config.clone()).start();
    let parent = p.start();

    let listen_result = s
        .send(Listen {
            parent: parent.clone().recipient(),
            evt_receiver: parent.recipient(),
        })
        .await;
    assert!(matches!(listen_result, Ok(Ok(..))));

    let addr = listen_result.unwrap().unwrap();
    assert!(addr.is_ipv4());

    let (_parts, body) = bsnext_utils::req_to_str(addr, "/", |a| {
        a.insert(ACCEPT, TEXT_HTML_UTF_8.to_string().parse().expect("s"));
        a
    })
    .await
    .expect("html response");
    assert_eq!(body, "hello world!".to_string());
    let (_parts, body) = bsnext_utils::req_to_str(addr, "/j", |a| {
        a.insert(ACCEPT, APPLICATION_JSON.to_string().parse().expect("s"));
        a
    })
    .await
    .expect("html response");

    assert_eq!(body, "[]".to_string());
}

async fn system_test_02() {
    let route1 = Route {
        path: "/".to_string(),
        kind: RouteKind::new_html("hello world!"),
        ..Default::default()
    };
    let server_parent = ServerParent::from_routes(vec![route1]);
    let server_actor = ServerActor::new_from_config(server_parent.server_config.clone()).start();
    let parent = server_parent.start();

    let list_result = server_actor
        .send(Listen {
            parent: parent.clone().recipient(),
            evt_receiver: parent.clone().recipient(),
        })
        .await;
    assert!(matches!(list_result, Ok(Ok(..))));

    let addr = list_result.unwrap().unwrap();
    assert!(addr.is_ipv4());

    let json_payload = r#"
    {
      "kind": "Change",
      "payload": {
        "kind": "Fs",
        "payload": {
          "path": "styles.css",
          "change_kind": "Changed"
        }
      }
    }
    "#;

    let (_parts, body) = bsnext_utils::post_to_events(addr, json_payload)
        .await
        .expect("html response");

    assert_eq!(body, r#"{"ok":true}"#.to_string());

    let events = parent.send(GetEvents).await.unwrap();
    let first = events.get(0).unwrap();
    match first {
        IncomingEvents::FilesChanged(FilesChanged { paths, .. }) => {
            let first = paths.get(0).unwrap();
            assert_eq!(first, &PathBuf::from("styles.css"));
        }
    }
}

struct ServerParent {
    server_config: ServerConfig,
    events: Vec<IncomingEvents>,
}

impl ServerParent {
    pub fn from_routes(routes: Vec<Route>) -> Self {
        let server_config = ServerConfig {
            identity: ServerIdentity::named(),
            routes,
            watchers: vec![],
        };

        Self {
            server_config,
            events: vec![],
        }
    }
}

impl actix::Actor for ServerParent {
    type Context = actix::Context<Self>;
}

impl actix::Handler<GetServersMessage> for ServerParent {
    type Result = GetServersMessageResponse;

    fn handle(&mut self, _msg: GetServersMessage, _ctx: &mut Self::Context) -> Self::Result {
        todo!("woop!")
    }
}
impl actix::Handler<IncomingEvents> for ServerParent {
    type Result = ();

    fn handle(&mut self, msg: IncomingEvents, _ctx: &mut Self::Context) -> Self::Result {
        self.events.push(msg);
        ()
    }
}
#[derive(actix::Message)]
#[rtype(result = "Vec<IncomingEvents>")]
struct GetEvents;

impl actix::Handler<GetEvents> for ServerParent {
    type Result = Vec<IncomingEvents>;

    fn handle(&mut self, _msg: GetEvents, _ctx: &mut Self::Context) -> Self::Result {
        self.events.clone()
    }
}

#[actix_rt::test]
async fn test_init() {
    system_test_01().await
}

#[actix_rt::test]
async fn test_init_02() {
    system_test_02().await
}
