use axum::body::Body;
use axum::extract::{Request, State};
use axum::handler::Handler;
use axum::middleware::Next;
use axum::response::IntoResponse;
use axum::{middleware, Router};
use bsnext_input::route::{DirRoute, ProxyRoute, RawRoute, Route, RouteKind};
use http::{StatusCode, Uri};
use http_body_util::BodyExt;
use std::collections::HashMap;
use std::path::PathBuf;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use tracing::trace_span;

#[derive(Debug, Clone)]
pub struct ServeDirItem {
    pub path: PathBuf,
}

pub async fn try_many_serve_dir(
    State((items, root_path)): State<(Vec<ServeDirItem>, Option<PathBuf>)>,
    uri: Uri,
    r: Request,
    next: Next,
) -> impl IntoResponse {
    let span = trace_span!("handling");
    let _ = span.enter();
    tracing::trace!(?items);

    let svs = items
        .iter()
        .map(|item| {
            let src = match &root_path {
                Some(p) => {
                    tracing::trace!(
                        "combining root: `{}` with given path: `{}`",
                        p.display(),
                        item.path.display()
                    );
                    ServeDir::new(p.join(&item.path))
                }
                None => {
                    tracing::trace!("no root given, using `{}` directly", item.path.display());
                    ServeDir::new(&item.path)
                }
            };
            (item.path.clone(), src)
        })
        .collect::<Vec<(PathBuf, ServeDir)>>();

    tracing::trace!(?uri, "{} services", svs.len());

    let (a, b) = r.into_parts();

    for (index, (path_buf, srv)) in svs.into_iter().enumerate() {
        let span = trace_span!("trying {}", index);
        let _ = span.enter();
        tracing::trace!(?path_buf);
        let req_clone = Request::from_parts(a.clone(), Body::empty());
        let result = srv.oneshot(req_clone).await;
        match result {
            Ok(result) if result.status() == 404 => {
                tracing::trace!("  ❌ not found at index {}, trying another", index);
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
}

#[derive(Debug, PartialEq)]
pub enum HandlerStack {
    None,
    // todo: make this a separate thing
    Raw(RawRoute),
    Dirs(Vec<DirRoute>),
    Proxy(ProxyRoute),
    DirsProxy(Vec<DirRoute>, ProxyRoute),
}

pub fn routes_to_stack(state: HandlerStack, route: Route) -> HandlerStack {
    match state {
        HandlerStack::None => match route.kind {
            RouteKind::Raw(route) => HandlerStack::Raw(route),
            RouteKind::Proxy(pr) => HandlerStack::Proxy(pr),
            RouteKind::Dir(dir) => HandlerStack::Dirs(vec![dir]),
        },
        HandlerStack::Raw(..) => match route.kind {
            // if a second 'raw' is seen, just use it, discarding the previous
            RouteKind::Raw(route) => HandlerStack::Raw(route),
            // 'raw' handlers never get updated
            _ => state,
        },
        HandlerStack::Dirs(mut dirs) => match route.kind {
            RouteKind::Dir(next_dir) => {
                dirs.push(next_dir);
                HandlerStack::Dirs(dirs)
            }
            RouteKind::Proxy(proxy) => HandlerStack::DirsProxy(dirs, proxy),
            _ => HandlerStack::Dirs(dirs),
        },
        HandlerStack::Proxy(proxy) => match route.kind {
            RouteKind::Dir(dir) => HandlerStack::DirsProxy(vec![dir], proxy),
            _ => HandlerStack::Proxy(proxy),
        },
        HandlerStack::DirsProxy(mut dirs, proxy) => match route.kind {
            RouteKind::Dir(dir) => {
                dirs.push(dir);
                HandlerStack::DirsProxy(dirs, proxy)
            }
            _ => HandlerStack::DirsProxy(dirs, proxy),
        },
    }
}

#[test]
fn test_route_stack() -> anyhow::Result<()> {
    let routes = r#"
    - path: /dir1
      dir: 'another'
    - path: /dir1
      dir: 'another_2'
    - path: /dir1
      proxy: 'example.com'
    "#;
    let routes = serde_yaml::from_str::<Vec<Route>>(&routes)?;

    let output = routes
        .into_iter()
        .fold(HandlerStack::None, |s, route| routes_to_stack(s, route));

    dbg!(&output);

    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::server::router::common::to_resp_parts_and_body;
    use std::env::current_dir;
    #[tokio::test]
    async fn test() -> anyhow::Result<()> {
        let routes_input = r#"
            - path: /
              dir: examples/basic/public
            - path: /
              dir: examples/kitchen-sink
        "#;

        {
            let current = current_dir()?;
            let parent = current.parent().unwrap().parent().unwrap().to_owned();
            let routes = serde_yaml::from_str::<Vec<Route>>(&routes_input)?;
            let router = create_dir_router(&routes, Some(parent));
            let expected_body = include_str!("../../../examples/basic/public/index.html");

            // Define the request
            let request = Request::get("/index.html").body(Body::empty())?;
            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;
            let (parts, actual_body) = to_resp_parts_and_body(response).await;
            assert_eq!(actual_body, expected_body);
        }

        {
            let current = current_dir()?;
            let parent = current.parent().unwrap().parent().unwrap().to_owned();
            let routes = serde_yaml::from_str::<Vec<Route>>(&routes_input)?;
            let router = create_dir_router(&routes, Some(parent));
            let expected_body = include_str!("../../../examples/kitchen-sink/input.html");

            // Define the request
            let request = Request::get("/input.html").body(Body::empty())?;
            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;
            let (parts, actual_body) = to_resp_parts_and_body(response).await;
            assert_eq!(actual_body, expected_body);
        }

        Ok(())
    }
}

pub fn create_dir_router(routes: &[Route], root_path: Option<PathBuf>) -> Router {
    let route_map = routes
        .iter()
        .filter(|r| matches!(&r.kind, RouteKind::Dir(_)))
        .fold(HashMap::<String, Vec<Route>>::new(), |mut acc, route| {
            acc.entry(route.path.clone())
                .and_modify(|acc| acc.push(route.clone()))
                .or_insert(vec![route.clone()]);
            acc
        });

    let mut router = Router::new();
    for (path, route_list) in route_map {
        tracing::trace!("register {} routes for path {}", route_list.len(), path);
        let serve_dir_items = route_list.iter().filter_map(route_to_serve_dir).collect();

        let temp_router = Router::new().layer(middleware::from_fn_with_state(
            (serve_dir_items, root_path.to_owned()),
            try_many_serve_dir,
        ));
        router = router.nest_service(&path, temp_router.into_service());
    }
    router
}

fn route_to_serve_dir(r: &Route) -> Option<ServeDirItem> {
    match &r.kind {
        RouteKind::Dir(dir) => Some(ServeDirItem {
            path: PathBuf::from(&dir.dir),
        }),
        _ => None,
    }
}
