use actix::Actor;
use axum::body::Body;
use bsnext_core::server::actor::ServerActor;
use bsnext_core::server::handler_listen::Listen;
use bsnext_core::servers_supervisor::get_servers_handler::GetServersMessage;
use bsnext_dto::GetServersMessageResponse;
use bsnext_input::route::{JsonWrapper, Route, RouteKind};
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use http::header::ACCEPT;
use http::response::Parts;
use http::{HeaderMap, Request, Uri};
use http_body_util::BodyExt;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use mime_guess::mime::{APPLICATION_JSON, TEXT_HTML_UTF_8};
use serde_json::Value;
use std::net::SocketAddr;

async fn system_test_01() {
    let route1 = Route {
        path: "/".to_string(),
        kind: RouteKind::Html {
            html: "hello world!".to_string(),
        },
        ..Default::default()
    };
    let value: Value = serde_json::from_str("[]").expect("json");
    let route2 = Route {
        path: "/j".to_string(),
        kind: RouteKind::Json {
            json: JsonWrapper(value),
        },
        ..Default::default()
    };
    let p = ServerParent::from_routes(vec![route1, route2]);
    let s = ServerActor::new_from_config(p.server_config.clone()).start();
    let parent = p.start();

    let a = s
        .send(Listen {
            parent: parent.recipient(),
        })
        .await;

    match a {
        Ok(Ok(addr)) => {
            assert!(addr.is_ipv4());
            let (_parts, body) = request_str(addr, "/", |a| {
                a.insert(ACCEPT, TEXT_HTML_UTF_8.to_string().parse().expect("s"));
                a
            })
            .await
            .expect("html response");
            assert_eq!(body, "hello world!".to_string());
            let (_parts, body) = request_str(addr, "/j", |a| {
                a.insert(ACCEPT, APPLICATION_JSON.to_string().parse().expect("s"));
                a
            })
            .await
            .expect("html response");

            assert_eq!(body, "[]".to_string());
        }
        Ok(Err(e)) => {
            unreachable!("{:?}", e)
        }
        Err(e) => unreachable!("{:?}", e),
    }
}

async fn request_str(
    socket_addr: SocketAddr,
    uri: &str,
    headers: fn(&mut HeaderMap) -> &mut HeaderMap,
) -> anyhow::Result<(Parts, String)> {
    let https = HttpsConnector::new();
    let client: Client<HttpsConnector<HttpConnector>, Body> =
        Client::builder(TokioExecutor::new()).build(https);

    let uri = Uri::builder()
        .scheme("http")
        .authority(socket_addr.to_string())
        .path_and_query(uri)
        .build()
        .expect("valid uri");

    let mut r = Request::builder().uri(uri).body(Body::empty()).unwrap();
    headers(r.headers_mut());

    let resp = client.request(r).await.expect("result");

    let (parts, body) = resp.into_parts();

    let bytes = match body.collect().await {
        Ok(c) => c.to_bytes(),
        Err(_) => unreachable!("cannot error"),
    };

    match std::str::from_utf8(&bytes[..]) {
        Ok(s) => Ok((parts, String::from(s))),
        Err(_e) => Err(anyhow::anyhow!("oops")),
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
            watchers: vec![],
        };

        Self { server_config }
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

#[actix_rt::test]
async fn test_init() {
    system_test_01().await
}
