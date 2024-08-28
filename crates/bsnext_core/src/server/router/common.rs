use crate::server::router::make_router;
use crate::server::state::ServerState;
use std::net::SocketAddr;

use axum::body::Body;
use axum::extract::Request;
use axum::response::Response;
use axum::Router;
use bsnext_dto::ClientEvent;
use bsnext_input::server_config::ServerConfig;
use bsnext_input::Input;
use http::header::ACCEPT;
use http::response::Parts;
use http::HeaderValue;
use mime_guess::mime::TEXT_HTML;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tower::ServiceExt;

pub fn into_state(val: ServerConfig) -> ServerState {
    let (sender, _) = tokio::sync::broadcast::channel::<ClientEvent>(10);
    ServerState {
        routes: Arc::new(RwLock::new(val.routes.clone())),
        raw_router: Arc::new(RwLock::new(Router::new())),
        id: val.identity.as_id(),
        parent: None,
        evt_receiver: None,
        client_sender: Arc::new(sender),
    }
}

pub async fn to_resp_body(res: Response) -> String {
    use http_body_util::BodyExt;
    let (_parts, body) = res.into_parts();
    let b = body.collect().await.unwrap();
    let b = b.to_bytes();
    let as_str = std::str::from_utf8(&b).unwrap();
    as_str.to_owned()
}

pub async fn to_resp_parts_and_body(res: Response) -> (Parts, String) {
    use http_body_util::BodyExt;
    let (parts, body) = res.into_parts();
    let b = body.collect().await.unwrap();
    let b = b.to_bytes();
    let as_str = std::str::from_utf8(&b).unwrap();
    (parts, as_str.to_owned())
}

pub async fn req_to_body(state: ServerState, uri: &str) -> String {
    let app = make_router(&Arc::new(state));
    let req = Request::get(uri).body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    to_resp_body(res).await
}

pub async fn accept_html_req_to_body(state: ServerState, uri: &str) -> String {
    let app = make_router(&Arc::new(state));
    let mut req = Request::get(uri).body(Body::empty()).unwrap();
    req.headers_mut()
        .insert(ACCEPT, HeaderValue::from_str(TEXT_HTML.as_ref()).unwrap());
    let res = app.oneshot(req).await.unwrap();
    to_resp_body(res).await
}

pub async fn uri_to_res(state: ServerState, uri: &str) -> Response {
    let app = make_router(&Arc::new(state));
    let req = Request::get(uri).body(Body::empty()).unwrap();
    app.oneshot(req).await.unwrap()
}

pub async fn uri_to_res_parts(state: ServerState, uri: &str) -> (Parts, String, Duration) {
    let app = make_router(&Arc::new(state));
    let req = Request::get(uri).body(Body::empty()).unwrap();
    let start = Instant::now();
    let res = app.oneshot(req).await.unwrap();
    let end = Instant::now();
    let diff = end - start;
    let (parts, body) = to_resp_parts_and_body(res).await;
    (parts, body, diff)
}

pub fn from_yaml(yaml: &str) -> anyhow::Result<ServerState> {
    let input: Input = serde_yaml::from_str(yaml)?;
    let config: ServerConfig = input.servers.first().expect("first").to_owned();
    let state = into_state(config);
    Ok(state)
}

pub struct TestProxy {
    pub socker_addr: SocketAddr,
    pub http_addr: String,
    pub shutdown: tokio::sync::oneshot::Sender<()>,
    pub join_handle: JoinHandle<()>,
}

impl TestProxy {
    pub async fn destroy(self) -> anyhow::Result<()> {
        self.shutdown.send(()).expect("did send");
        self.join_handle.await?;
        Ok(())
    }
}

pub async fn test_proxy(router: Router) -> anyhow::Result<TestProxy> {
    let (address_sender, address_receiver) = tokio::sync::oneshot::channel::<SocketAddr>();
    let (complete_sender, complete_receiver) = tokio::sync::oneshot::channel::<()>();

    let handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let socket_addr = listener.local_addr().unwrap();

        // give consumers the address
        address_sender.send(socket_addr).expect("can send");

        // serve and wait for shutdown
        axum::serve(listener, router)
            .with_graceful_shutdown(async {
                complete_receiver.await.expect("TODO: panic message");
            })
            .await
            .unwrap();
    });

    let addr = address_receiver.await?;
    let http_address = format!("http://{addr}");
    Ok(TestProxy {
        socker_addr: addr,
        http_addr: http_address,
        shutdown: complete_sender,
        join_handle: handle,
    })
}
