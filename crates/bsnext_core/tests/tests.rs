use axum::body::Body;
use axum::extract::Request;
use bsnext_core::server::router::common::{from_yaml, into_state, to_resp_body, uri_to_res_parts};
use bsnext_core::server::router::make_router;
use bsnext_input::route::{CorsOpts, Opts, Route, RouteKind};
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use http::HeaderValue;
use std::collections::BTreeMap;
use std::sync::Arc;
use tower::ServiceExt;

#[tokio::test]
async fn test_handlers() -> Result<(), anyhow::Error> {
    let input = r#"
    servers:
      - bind_address: 0.0.0.0:3000
        routes:
          - path: /hello
            html: "🐥"
            headers:
                a: b
    "#;
    let state = from_yaml(input)?;
    let (parts, body, ..) = uri_to_res_parts(state, "/hello").await;

    insta::assert_debug_snapshot!(parts.headers);

    assert_eq!(body, "🐥");
    Ok(())
}

#[tokio::test]
async fn test_handlers_raw() -> Result<(), anyhow::Error> {
    let input = r#"
    servers:
      - bind_address: 0.0.0.0:3000
        routes:
          - path: /styles.css
            raw: "body{}"
    "#;
    let state = from_yaml(input)?;
    let (parts, body, ..) = uri_to_res_parts(state, "/styles.css").await;

    assert_eq!(parts.headers.get("content-length").unwrap(), "6");
    assert_eq!(parts.headers.get("content-type").unwrap(), "text/css");
    assert_eq!(body, "body{}");

    Ok(())
}

#[tokio::test]
async fn test_cors_handlers() -> Result<(), anyhow::Error> {
    let input = r#"
    servers:
      - bind_address: 0.0.0.0:3000
        routes:
          - path: /
            html: home
            cors: true
    "#;
    let state = from_yaml(input)?;
    let (parts, body, ..) = uri_to_res_parts(state, "/").await;
    let h = parts.headers.get("access-control-allow-origin");
    let v = parts.headers.get("vary");

    assert_eq!(h, Some(HeaderValue::from_str("*").as_ref().unwrap()));
    assert_eq!(
        v,
        Some(
            HeaderValue::from_str(
                "origin, access-control-request-method, access-control-request-headers"
            )
            .as_ref()
            .unwrap()
        )
    );

    assert_eq!(body, "home");
    Ok(())
}
async fn test_not_found_handler() -> Result<(), anyhow::Error> {
    let input = r#"
    servers:
      - bind_address: 0.0.0.0:3000
        routes:
          - path: /hello
            html: home
    "#;
    let state = from_yaml(&input)?;
    let app = make_router(&Arc::new(state));

    let req = Request::get("/abc").body(Body::empty()).unwrap();
    let res = app.oneshot(req).await.unwrap();
    let status = res.status().as_u16();

    assert_eq!(
        res.headers().get("content-type").unwrap(),
        "text/html; charset=utf-8"
    );

    let body = to_resp_body(res).await;

    assert!(body.contains("<title>Browsersync LIVE</title>"));
    assert_eq!(status, 404);
    Ok(())
}
#[tokio::test]
async fn test_route_list() -> Result<(), anyhow::Error> {
    let input = r#"
    servers:
      - bind_address: 0.0.0.0:3000
        routes:
          - path: /abc
            html: home
    "#;
    let state = from_yaml(input)?;
    let (parts, body, ..) = uri_to_res_parts(state, "/__bslive").await;

    let status = parts.status.as_u16();

    assert_eq!(
        parts.headers.get("content-type").unwrap(),
        "text/html; charset=utf-8"
    );

    assert!(body.contains("<title>Browsersync LIVE</title>"));
    assert!(body.contains("<base href=\"/__bs_assets/ui/\" />"));
    assert_eq!(status, 200);
    Ok(())
}
