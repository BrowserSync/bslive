#[cfg(test)]
mod test {

    use crate::handler_stack::RouteMap;
    use crate::server::router::common::to_resp_parts_and_body;

    use crate::runtime_ctx::RuntimeCtx;
    use axum::body::Body;
    use axum::extract::Request;
    use bsnext_input::route::Route;
    use std::env::current_dir;
    use tower::ServiceExt;

    #[tokio::test]
    async fn test() -> anyhow::Result<()> {
        let current = current_dir()?;
        let parent = current.parent().unwrap().parent().unwrap().to_owned();

        let routes_input = format!(
            r#"
            - path: /
              dir: examples/basic/public
              base: {base}
            - path: /
              dir: examples/kitchen-sink
              base: {base}
        "#,
            base = parent.display()
        );

        let routes = serde_yaml::from_str::<Vec<Route>>(&routes_input)?;

        {
            let router = RouteMap::new_from_routes(&routes).into_router(&RuntimeCtx::default());
            let expected_body = include_str!("../../../examples/basic/public/index.html");

            // Define the request
            let request = Request::get("/index.html").body(Body::empty())?;
            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;
            let (_parts, actual_body) = to_resp_parts_and_body(response).await;
            assert_eq!(actual_body, expected_body);
        }

        {
            let router = RouteMap::new_from_routes(&routes).into_router(&RuntimeCtx::default());
            let expected_body = include_str!("../../../examples/kitchen-sink/input.html");

            // Define the request
            let request = Request::get("/input.html").body(Body::empty())?;
            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;
            let (_parts, actual_body) = to_resp_parts_and_body(response).await;
            assert_eq!(actual_body, expected_body);
        }

        Ok(())
    }
}
