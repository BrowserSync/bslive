use axum::routing::get;
use axum::Router;
use bsnext_core::server::router::common::{
    accept_html_req_to_body, into_state, test_proxy, uri_to_res, TestProxy,
};
use bsnext_core::server::state::ServerState;
use bsnext_input::server_config::ServerConfig;
use bsnext_input::Input;
use http::header::CONTENT_TYPE;
use insta::{assert_debug_snapshot, assert_snapshot};

#[tokio::test]
async fn route_kinds_minus_proxy() -> Result<(), anyhow::Error> {
    let input = format!(
        r#"
servers:
- bind_address: 127.0.0.1:9000
  routes:
  - path: /
    html: <body>HTML route kind</body>
  - path: /json
    json: {{ a: {{ b: [1, 2] }} }}
  - path: /raw
    raw: -----raw-content----
  - path: /sse
    sse: |
        a
        b
        c
  - path: /dir
    dir: .
    "#
    );
    let state = from_yaml(&input)?;

    let body = accept_html_req_to_body(state.clone(), "/").await;
    assert_snapshot!(body);

    let body2 = accept_html_req_to_body(state.clone(), "/json").await;
    assert_snapshot!(body2);

    let body3 = accept_html_req_to_body(state.clone(), "/raw").await;
    assert_snapshot!(body3);

    let body4 = accept_html_req_to_body(state.clone(), "/sse").await;
    assert_snapshot!(body4);

    let res = uri_to_res(state.clone(), "/dir/Cargo.toml").await;
    let status = res.status();
    let header = res.headers().get(CONTENT_TYPE);
    assert_debug_snapshot!((status, header));

    Ok(())
}

#[tokio::test]
async fn test_proxy_route() -> Result<(), anyhow::Error> {
    let proxy_app = Router::new()
        .route("/", get(|| async { "target!" }))
        .route("/something-else", get(|| async { "target other!" }));

    let proxy = test_proxy(proxy_app).await?;
    let TestProxy { ref http_addr, .. } = proxy;

    let input = format!(
        r#"
servers:
- bind_address: 127.0.0.1:9000
  routes:
  - path: /
    proxy: {http_addr}
    "#
    );

    let state = from_yaml(&input)?;

    let body1 = accept_html_req_to_body(state.clone(), "/").await;
    let body2 = accept_html_req_to_body(state.clone(), "/something-else").await;

    proxy.destroy().await?;

    assert_eq!(body1, "target!");
    assert_eq!(body2, "target other!");

    Ok(())
}

fn from_yaml(yaml: &str) -> anyhow::Result<ServerState> {
    let input: Input = serde_yaml::from_str(yaml)?;
    let config: ServerConfig = input.servers.get(0).expect("first").to_owned();
    let state = into_state(config);
    Ok(state)
}
