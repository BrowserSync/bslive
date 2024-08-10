use bsnext_core::server::router::common::{accept_html_req_to_body, into_state, req_to_body};
use bsnext_core::server::state::ServerState;
use bsnext_input::server_config::ServerConfig;
use bsnext_input::Input;
use insta::assert_snapshot;

fn yaml_server_01() -> ServerState {
    let input = r#"
servers:
- bind_address: 127.0.0.1:9000
  routes:
  - path: /
    html: <body>hey!</body>
  - path: /other
    html: <body>hey from other!</body>
  - path: /form.html
    html: <body>Should be excluded, since the `only` doesn't match</body>
    inject:
      name: bslive:connector
      only: /index.html
  - path: /app.js
    raw: console.log("test!")
  - path: /styles.css
    raw: 'body{}'
    inject:
      - append: lol
        only: '/*.css'
    "#;
    let input: Input = serde_yaml::from_str(input).expect("input");
    let config: ServerConfig = input.servers.get(0).expect("fiirst").to_owned();
    let state = into_state(config).into();
    state
}

#[tokio::test]
async fn test_handlers_raw_inject() -> Result<(), anyhow::Error> {
    let state = yaml_server_01();
    let body = req_to_body(state, "/styles.css").await;
    assert_eq!(body, "body{}lol");

    // with param
    let state = yaml_server_01();
    let body = req_to_body(state, "/styles.css?oops=does_not_affect").await;
    assert_eq!(body, "body{}lol");
    Ok(())
}

#[tokio::test]
async fn test_handlers_raw_inject_js() -> Result<(), anyhow::Error> {
    let state = yaml_server_01();
    let body = req_to_body(state, "/app.js").await;
    assert_eq!(body, "console.log(\"test!\")", "unchanged request");
    Ok(())
}

#[tokio::test]
async fn overriding_built_in() -> Result<(), anyhow::Error> {
    let state = yaml_server_01();
    let body = accept_html_req_to_body(state, "/").await;
    assert_snapshot!(body);

    let state = yaml_server_01();
    let body2 = accept_html_req_to_body(state, "/other").await;
    assert_snapshot!(body2);

    let state = yaml_server_01();
    let body3 = accept_html_req_to_body(state, "/form.html").await;
    assert_snapshot!(body3);
    Ok(())
}
