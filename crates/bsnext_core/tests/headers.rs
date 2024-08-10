use bsnext_core::server::router::common::{from_yaml, uri_to_res_parts};
use insta::assert_debug_snapshot;

#[tokio::test]
async fn test_headers() -> Result<(), anyhow::Error> {
    let input = include_str!("../../../examples/basic/headers.yml");
    let state = from_yaml(&input)?;

    let (parts1, body1, _) = uri_to_res_parts(state.clone(), "/").await;
    let (parts2, body2, _) = uri_to_res_parts(state.clone(), "/other").await;

    assert_debug_snapshot!((parts1.headers, body1));
    assert_debug_snapshot!((parts2.headers, body2));

    Ok(())
}
