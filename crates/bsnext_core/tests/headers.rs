use bsnext_core::server::router::common::{
    accept_html_req_to_body, from_yaml, into_state, test_proxy, to_resp_parts_and_body, uri_to_res,
    uri_to_res_parts, TestProxy,
};
use bsnext_core::server::state::ServerState;
use bsnext_input::server_config::ServerConfig;
use bsnext_input::Input;
use insta::assert_debug_snapshot;

#[tokio::test]
async fn test_headers() -> Result<(), anyhow::Error> {
    let input = include_str!("../../../examples/basic/headers.yml");
    let state = from_yaml(&input)?;

    let (parts1, body1) = uri_to_res_parts(state.clone(), "/").await;
    let (parts2, body2) = uri_to_res_parts(state.clone(), "/other").await;

    assert_debug_snapshot!((parts1.headers, body1));
    assert_debug_snapshot!((parts2.headers, body2));

    Ok(())
}
