use axum::routing::get;
use axum::Router;
use bsnext_core::server::router::common::{from_yaml, into_state, test_proxy, uri_to_res_parts};
use bsnext_input::route::{ProxyRoute, RouteKind};
use bsnext_input::server_config::ServerConfig;
use bsnext_input::Input;
use insta::assert_debug_snapshot;

#[tokio::test]
async fn test_delays() -> Result<(), anyhow::Error> {
    let input = include_str!("../../../examples/basic/delays.yml");
    let state = from_yaml(&input)?;

    let (parts1, body1, dur1) = uri_to_res_parts(state.clone(), "/").await;
    let (parts2, body2, dur2) = uri_to_res_parts(state.clone(), "/500").await;
    let (_, _, dur3) = uri_to_res_parts(state.clone(), "/dir").await;

    let dur1_ms = dur1.as_millis();
    let dur2_ms = dur2.as_millis();
    let dur3_ms = dur3.as_millis();

    assert!(dur1_ms > 200 && dur1_ms < 210);
    assert!(dur2_ms > 500 && dur2_ms < 510);
    assert!(
        dur3_ms > 300 && dur3_ms < 310,
        "dir delay should be over 300ms"
    );

    assert_debug_snapshot!((parts1.headers, body1));
    assert_debug_snapshot!((parts2.headers, body2));

    Ok(())
}

#[tokio::test]
async fn test_proxy_delay() -> Result<(), anyhow::Error> {
    let proxy_app = Router::new().route("/", get(|| async { "target - proxy delay" }));
    let proxy = test_proxy(proxy_app).await?;

    // read from yaml
    let yaml = include_str!("../../../examples/basic/delays.yml");
    let input: Input = serde_yaml::from_str(yaml)?;

    // update the proxy target to use local test proxy
    let mut config: ServerConfig = input.servers.first().expect("first").to_owned();
    let proxy_route = config.routes.get_mut(3).unwrap();
    assert!(
        matches!(proxy_route.kind, RouteKind::Proxy(..)),
        "must be a proxy route, check delays.yml"
    );
    proxy_route.kind = RouteKind::Proxy(ProxyRoute {
        proxy: proxy.http_addr.clone(),
    });

    let state = into_state(config);
    let (_, body1, dur1) = uri_to_res_parts(state.clone(), "/api").await;
    let millis = dur1.as_millis();

    // cleanup
    proxy.destroy().await?;

    // assertions
    assert!(millis > 300 && millis < 310);
    assert_eq!(body1, "target - proxy delay");
    Ok(())
}
