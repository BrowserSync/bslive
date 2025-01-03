use axum::body::Body;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::routing::MethodRouter;
use http::{StatusCode, Uri};
use tower::ServiceExt;

pub async fn try_many_services_dir(
    State(router_list): State<Vec<MethodRouter>>,
    uri: Uri,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    tracing::trace!(?uri, "{} services", router_list.len());

    let (a, b) = req.into_parts();

    for (index, method_router) in router_list.into_iter().enumerate() {
        // tracing::trace!(?path_buf);
        let req_clone = Request::from_parts(a.clone(), Body::empty());
        let result = method_router.oneshot(req_clone).await;
        match result {
            Ok(result) if result.status() == 404 => {
                tracing::trace!("  ❌ not found at index {}, trying another", index,);
                continue;
            }
            Ok(result) if result.status() == 405 => {
                tracing::trace!("  ❌ 405, trying another...");
                continue;
            }
            Ok(result) => {
                tracing::trace!(
                    ?index,
                    " - ✅ a non-404 response was given {}",
                    result.status()
                );
                return result.into_response();
            }
            Err(e) => {
                tracing::error!(?e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }
    tracing::trace!(" - REQUEST was NOT HANDLED BY SERVE_DIR (will be sent onwards)");
    let r = Request::from_parts(a.clone(), b);
    next.run(r).await
    // StatusCode::NOT_FOUND.into_response()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::handler_stack::RouteMap;
    use crate::server::router::common::to_resp_parts_and_body;

    use crate::runtime_ctx::RuntimeCtx;
    use bsnext_input::route::Route;
    use std::env::current_dir;

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
