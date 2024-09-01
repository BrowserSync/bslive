#[cfg(test)]
mod test {

    use crate::handler_stack::RouteMap;
    use crate::server::router::common::{test_proxy, to_resp_parts_and_body};
    use axum::body::Body;
    use axum::extract::Request;
    use axum::routing::get;
    use axum::{Extension, Router};
    use bsnext_input::route::Route;
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
            let router = RouteMap::new_from_routes(&routes)
                .into_router()
                .layer(Extension(client));

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
