use crate::handlers::proxy::{proxy_handler, ProxyConfig};
use axum::routing::any;
use axum::{Extension, Router};
use bsnext_input::route::{Route, RouteKind};
use std::collections::HashMap;

pub fn create_proxy_router(routes: &[Route]) -> Router {
    let route_map = routes
        .iter()
        .filter(|r| matches!(&r.kind, RouteKind::Proxy(_)))
        .fold(HashMap::<String, Vec<Route>>::new(), |mut acc, route| {
            acc.entry(route.path.clone())
                .and_modify(|acc| acc.push(route.clone()))
                .or_insert(vec![route.clone()]);
            acc
        });

    let mut router = Router::new();
    for (path, route_list) in route_map {
        tracing::trace!("register {} routes for path {}", route_list.len(), path);
        let r = route_list.last().unwrap();
        let proxy_config = route_to_proxy(r).unwrap();
        router = router.nest_service(
            &path,
            any(proxy_handler).layer(Extension(proxy_config.clone())),
        )
    }
    router
}

fn route_to_proxy(r: &Route) -> Option<ProxyConfig> {
    match &r.kind {
        RouteKind::Proxy(proxy) => Some(ProxyConfig {
            target: proxy.proxy.clone(),
            path: r.path.to_string(),
        }),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::server::router::common::{test_proxy, to_resp_parts_and_body};
    use axum::body::Body;
    use axum::extract::Request;
    use axum::routing::get;
    use hyper_tls::HttpsConnector;
    use hyper_util::client::legacy::connect::HttpConnector;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test() -> anyhow::Result<()> {
        let https = HttpsConnector::new();
        let client: Client<HttpsConnector<HttpConnector>, Body> =
            Client::builder(TokioExecutor::new()).build(https);

        let proxy_app = Router::new()
            .route("/", get(|| async { "target!" }))
            .route("/something-else", get(|| async { "target other!" }));

        let proxy = test_proxy(proxy_app).await?;

        let routes_input = format!(
            r#"
            - path: /
              proxy: {http}
        "#,
            http = proxy.http_addr
        );

        {
            let routes = serde_yaml::from_str::<Vec<Route>>(&routes_input)?;
            let router = create_proxy_router(&routes).layer(Extension(client));

            let expected_body = "target!";

            // Define the request
            let request = Request::get("/").body(Body::empty())?;

            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;

            let (parts, actual_body) = to_resp_parts_and_body(response).await;

            assert_eq!(actual_body, expected_body);
        }

        proxy.destroy().await?;

        Ok(())
    }
}
