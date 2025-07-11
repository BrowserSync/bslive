#[cfg(test)]
mod test {

    use crate::handler_stack::RouteMap;
    use crate::runtime_ctx::RuntimeCtx;
    use crate::server::router::common::{test_proxy, to_resp_parts_and_body};
    use axum::body::Body;
    use axum::extract::Request;
    use axum::routing::{get, post};
    use axum::{Extension, Json, Router};
    use bsnext_input::route::Route;

    use hyper_tls::HttpsConnector;
    use hyper_util::client::legacy::connect::HttpConnector;
    use hyper_util::client::legacy::Client;
    use hyper_util::rt::TokioExecutor;
    use serde_json::json;
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
                .into_router(&RuntimeCtx::default())
                .layer(Extension(client));

            let expected_body = "target!";

            // Define the request
            let request = Request::get("/").body(Body::empty())?;

            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;

            let (_parts, actual_body) = to_resp_parts_and_body(response).await;

            assert_eq!(actual_body, expected_body);
        }

        proxy.destroy().await?;

        Ok(())
    }
    #[tokio::test]
    async fn test_post() -> anyhow::Result<()> {
        let https = HttpsConnector::new();
        let client: Client<HttpsConnector<HttpConnector>, Body> =
            Client::builder(TokioExecutor::new()).build(https);

        #[derive(Debug, serde::Serialize, serde::Deserialize)]
        struct Person {
            name: String,
        }

        let proxy_app = Router::new().route(
            "/did-post",
            post(|Json(mut person): Json<Person>| async move {
                dbg!(&person);
                person.name = format!("-->{}", person.name);
                Json(person)
            }),
        );

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
                .into_router(&RuntimeCtx::default())
                .layer(Extension(client));

            let expected_response_body = "{\"name\":\"-->shane\"}";

            let outgoing_body = serde_json::to_string(&json!({
                "name": "shane"
            }))?;

            // Define the request
            let request = Request::post("/did-post")
                .header("Content-Type", "application/json")
                .body(outgoing_body)?;

            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;

            let (_parts, actual_body) = to_resp_parts_and_body(response).await;
            assert_eq!(actual_body, expected_response_body);
        }

        proxy.destroy().await?;

        Ok(())
    }
    #[tokio::test]
    async fn test_path_rewriting() -> anyhow::Result<()> {
        let https = HttpsConnector::new();
        let client: Client<HttpsConnector<HttpConnector>, Body> =
            Client::builder(TokioExecutor::new()).build(https);

        let proxy_app = Router::new()
            .route("/", get(|| async { "did rewrite" }))
            .route("/no-rewrite", get(|| async { "did not rewrite" }))
            .route("/api", get(|| async { "api" }))
            .route("/api/rewrite-alt-append", get(|| async { "api+appended" }))
            .route(
                "/a/b/nested/no-rewrite",
                get(|| async { "did not rewrite (nested)" }),
            );

        let proxy = test_proxy(proxy_app).await?;

        let routes_input = format!(
            r#"
            - path: /rewrite
              proxy: {http}
            - path: /no-rewrite
              proxy: {http}
              rewrite_uri: false
            - path: /a/b/nested/rewrite
              proxy: {http}
            - path: /a/b/nested/no-rewrite
              proxy: {http}
              rewrite_uri: false


            - path: /rewrite-alt
              proxy: {http}/api
            - path: /rewrite-alt-append
              proxy: {http}/api
              rewrite_uri: false
        "#,
            http = proxy.http_addr
        );

        {
            let routes = serde_yaml::from_str::<Vec<Route>>(&routes_input)?;
            let router = RouteMap::new_from_routes(&routes)
                .into_router(&RuntimeCtx::default())
                .layer(Extension(client.clone()));

            let expected_body = "did rewrite";

            let request = Request::get("/rewrite").body(Body::empty())?;
            let response = router.oneshot(request).await?;
            let (_parts, actual_body) = to_resp_parts_and_body(response).await;

            assert_eq!(actual_body, expected_body);
        }
        {
            let routes = serde_yaml::from_str::<Vec<Route>>(&routes_input)?;
            let router = RouteMap::new_from_routes(&routes)
                .into_router(&RuntimeCtx::default())
                .layer(Extension(client.clone()));

            let expected_body = "did not rewrite";

            let request = Request::get("/no-rewrite").body(Body::empty())?;
            let response = router.oneshot(request).await?;
            let (_parts, actual_body) = to_resp_parts_and_body(response).await;

            assert_eq!(actual_body, expected_body);
        }
        {
            let routes = serde_yaml::from_str::<Vec<Route>>(&routes_input)?;
            let router = RouteMap::new_from_routes(&routes)
                .into_router(&RuntimeCtx::default())
                .layer(Extension(client.clone()));

            let expected_body = "did rewrite";

            let request = Request::get("/a/b/nested/rewrite").body(Body::empty())?;
            let response = router.oneshot(request).await?;
            let (_parts, actual_body) = to_resp_parts_and_body(response).await;

            assert_eq!(actual_body, expected_body);
        }
        {
            let routes = serde_yaml::from_str::<Vec<Route>>(&routes_input)?;
            let router = RouteMap::new_from_routes(&routes)
                .into_router(&RuntimeCtx::default())
                .layer(Extension(client.clone()));

            let expected_body = "did not rewrite (nested)";

            let request = Request::get("/a/b/nested/no-rewrite").body(Body::empty())?;
            let response = router.oneshot(request).await?;
            let (_parts, actual_body) = to_resp_parts_and_body(response).await;

            assert_eq!(actual_body, expected_body);
        }

        {
            let routes = serde_yaml::from_str::<Vec<Route>>(&routes_input)?;
            let router = RouteMap::new_from_routes(&routes)
                .into_router(&RuntimeCtx::default())
                .layer(Extension(client.clone()));

            let expected_body = "api";

            let request = Request::get("/rewrite-alt").body(Body::empty())?;
            let response = router.oneshot(request).await?;
            let (_parts, actual_body) = to_resp_parts_and_body(response).await;

            assert_eq!(actual_body, expected_body);
        }

        {
            let routes = serde_yaml::from_str::<Vec<Route>>(&routes_input)?;
            let router = RouteMap::new_from_routes(&routes)
                .into_router(&RuntimeCtx::default())
                .layer(Extension(client.clone()));

            let expected_body = "api+appended";

            let request = Request::get("/rewrite-alt-append").body(Body::empty())?;
            let response = router.oneshot(request).await?;
            let (_parts, actual_body) = to_resp_parts_and_body(response).await;

            assert_eq!(actual_body, expected_body);
        }

        proxy.destroy().await?;

        Ok(())
    }
}
