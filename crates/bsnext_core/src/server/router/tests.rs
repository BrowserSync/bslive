#[cfg(test)]
pub mod common {
    use crate::server::router::make_router;
    use crate::server::state::ServerState;
    use axum::body::Body;
    use axum::extract::Request;
    use axum::response::Response;
    use bsnext_dto::ClientEvent;
    use bsnext_input::server_config::ServerConfig;
    use http::header::ACCEPT;
    use http::HeaderValue;
    use mime_guess::mime::TEXT_HTML;
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use tower::ServiceExt;

    impl From<ServerConfig> for ServerState {
        fn from(val: ServerConfig) -> ServerState {
            let (sender, _) = tokio::sync::broadcast::channel::<ClientEvent>(10);
            ServerState {
                routes: Arc::new(RwLock::new(val.routes.clone())),
                id: val.identity.as_id(),
                parent: None,
                client_sender: Arc::new(sender),
            }
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

    pub async fn req_to_body(state: ServerState, uri: &str) -> String {
        let app = make_router(&Arc::new(state));
        let req = Request::get(uri).body(Body::empty()).unwrap();
        let res = app.oneshot(req).await.unwrap();
        let body = to_resp_body(res).await;
        body
    }

    pub async fn accept_html_req_to_body(state: ServerState, uri: &str) -> String {
        let app = make_router(&Arc::new(state));
        let mut req = Request::get(uri).body(Body::empty()).unwrap();
        req.headers_mut().insert(
            ACCEPT,
            HeaderValue::from_str(&TEXT_HTML.to_string()).unwrap(),
        );
        let res = app.oneshot(req).await.unwrap();
        let body = to_resp_body(res).await;
        body
    }
}

#[cfg(test)]
mod test {
    use crate::server::router::make_router;
    use crate::server::router::tests::common::to_resp_body;
    use crate::server::state::ServerState;
    use axum::body::Body;
    use axum::extract::Request;
    
    
    use bsnext_input::route::{CorsOpts, Route, RouteKind};
    use bsnext_input::server_config::{Identity, ServerConfig};
    
    
    
    
    use http::HeaderValue;
    use std::collections::BTreeMap;
    use std::sync::Arc;
    
    use tower::ServiceExt;

    #[tokio::test]
    async fn test_handlers() -> Result<(), anyhow::Error> {
        let headers: BTreeMap<String, String> =
            [("a".to_string(), "b".to_string())].into_iter().collect();
        let state: ServerState = ServerConfig {
            identity: Identity::Address {
                bind_address: "127.0.0.1:3000".to_string(),
            },
            routes: vec![Route {
                path: "/hello".to_string(),
                kind: RouteKind::html("🐥"),
                headers: Some(headers),
                ..Default::default()
            }],
            ..Default::default()
        }
        .into();

        let app = make_router(&Arc::new(state));
        let req = Request::get("/hello").body(Body::empty()).unwrap();
        let res = app.oneshot(req).await.unwrap();

        insta::assert_debug_snapshot!(res.headers());
        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "text/html; charset=utf-8"
        );

        let body = to_resp_body(res).await;
        assert_eq!(body, "🐥");
        Ok(())
    }

    #[tokio::test]
    async fn test_handlers_raw() -> Result<(), anyhow::Error> {
        let state: ServerState = ServerConfig {
            identity: Identity::Address {
                bind_address: "127.0.0.1".to_string(),
            },
            routes: vec![Route {
                path: "/styles.css".to_string(),
                kind: RouteKind::Raw {
                    raw: "body{}".into(),
                },
                ..Default::default()
            }],
            ..Default::default()
        }
        .into();

        let app = make_router(&Arc::new(state));
        let req = Request::get("/styles.css").body(Body::empty()).unwrap();
        let res = app.oneshot(req).await.unwrap();

        assert_eq!(res.headers().get("content-length").unwrap(), "6");
        assert_eq!(res.headers().get("content-type").unwrap(), "text/css");

        let body = to_resp_body(res).await;
        assert_eq!(body, "body{}");
        Ok(())
    }

    #[tokio::test]
    async fn test_cors_handlers() -> Result<(), anyhow::Error> {
        let state: ServerState = ServerConfig {
            identity: Identity::Address {
                bind_address: "127.0.0.1:3000".to_string(),
            },
            routes: vec![
                Route {
                    path: "/".to_string(),
                    cors_opts: Some(CorsOpts::Cors(true)),
                    kind: RouteKind::html("home"),
                    ..Default::default()
                },
                Route {
                    path: "/hello".to_string(),
                    kind: RouteKind::html("🐥"),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }
        .into();

        let app = make_router(&Arc::new(state));
        let req = Request::get("/").body(Body::empty()).unwrap();
        let res = app.oneshot(req).await.unwrap();
        let h = res.headers().get("access-control-allow-origin");
        let v = res.headers().get("vary");

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

        let body = to_resp_body(res).await;
        assert_eq!(body, "home");
        Ok(())
    }
    #[tokio::test]
    async fn test_not_found_handler() -> Result<(), anyhow::Error> {
        let state: ServerState = ServerConfig {
            identity: Identity::Address {
                bind_address: "127.0.0.1:3000".to_string(),
            },
            routes: vec![Route {
                path: "/".to_string(),
                cors_opts: Some(CorsOpts::Cors(true)),
                kind: RouteKind::html("home"),
                ..Default::default()
            }],
            ..Default::default()
        }
        .into();

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
        let state: ServerState = ServerConfig {
            identity: Identity::Address {
                bind_address: "127.0.0.1:3000".to_string(),
            },
            routes: vec![Route {
                path: "/abc".to_string(),
                cors_opts: Some(CorsOpts::Cors(true)),
                kind: RouteKind::html("home"),
                ..Default::default()
            }],
            watchers: vec![],
        }
        .into();

        let app = make_router(&Arc::new(state));
        let req = Request::get("/__bslive").body(Body::empty()).unwrap();
        let res = app.oneshot(req).await.unwrap();
        let status = res.status().as_u16();

        assert_eq!(
            res.headers().get("content-type").unwrap(),
            "text/html; charset=utf-8"
        );

        let body = to_resp_body(res).await;

        assert!(body.contains("<title>Browsersync LIVE</title>"));
        assert!(body.contains("<base href=\"/__bs_assets/ui/\" />"));
        assert_eq!(status, 200);
        Ok(())
    }
}
