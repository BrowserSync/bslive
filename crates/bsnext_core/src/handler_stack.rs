use crate::handlers::proxy::{proxy_handler, ProxyConfig};
use crate::raw_loader::serve_raw_one;
use crate::serve_dir::{try_many_serve_dir, ServeDirItem};
use axum::handler::Handler;
use axum::routing::{any, any_service};
use axum::{middleware, Extension, Router};
use bsnext_input::route::{DirRoute, ProxyRoute, RawRoute, Route, RouteKind};
use std::collections::HashMap;
use std::path::PathBuf;
use tower::ServiceBuilder;
use tower_http::services::ServeDir;

#[derive(Debug, PartialEq)]
pub enum HandlerStack {
    None,
    // todo: make this a separate thing
    Raw(RawRoute),
    Dirs(Vec<DirRoute>),
    Proxy(ProxyRoute),
    DirsProxy(Vec<DirRoute>, ProxyRoute),
}

pub struct RouteMap {
    pub mapping: HashMap<String, Vec<Route>>,
}

impl RouteMap {
    pub fn new_from_routes(routes: &[Route]) -> Self {
        Self {
            mapping: routes
                .iter()
                .fold(HashMap::<String, Vec<Route>>::new(), |mut acc, route| {
                    acc.entry(route.path.clone())
                        .and_modify(|acc| acc.push(route.clone()))
                        .or_insert(vec![route.clone()]);
                    acc
                }),
        }
    }

    pub fn into_router(self) -> Router {
        let mut router = Router::new();

        tracing::trace!("processing `{}` different routes", self.mapping.len());

        for (path, route_list) in self.mapping {
            tracing::trace!(
                "processing path: `{}` with `{}` routes",
                path,
                route_list.len()
            );

            let stack = routes_to_stack(&route_list);
            let path_router = stack_to_router(&path, stack);

            tracing::trace!("will merge router at path: `{path}`");
            router = router.merge(path_router);
        }

        router
    }
}

pub fn append_stack(state: HandlerStack, route: Route) -> HandlerStack {
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

pub fn routes_to_stack(routes: &[Route]) -> HandlerStack {
    routes.iter().fold(HandlerStack::None, |s, route| {
        append_stack(s, route.clone())
    })
}

pub fn stack_to_router(path: &str, stack: HandlerStack) -> Router {
    match stack {
        HandlerStack::None => unreachable!(),
        HandlerStack::Raw(raw) => {
            let svc = any_service(serve_raw_one.with_state(raw));
            Router::new().route_service(path, ServiceBuilder::new().service(svc))
        }
        HandlerStack::Dirs(dir_list) => {
            Router::new().nest_service(path, serve_dir_layer(&dir_list, Router::new()))
        }
        HandlerStack::Proxy(proxy) => {
            let proxy_config = ProxyConfig {
                target: proxy.proxy.clone(),
                path: path.to_string(),
            };
            Router::new().nest_service(
                path,
                any(proxy_handler).layer(Extension(proxy_config.clone())),
            )
        }
        HandlerStack::DirsProxy(dir_list, proxy) => {
            let r2 = stack_to_router(path, HandlerStack::Proxy(proxy));
            let r1 = serve_dir_layer(&dir_list, Router::new().fallback_service(r2));
            Router::new().nest_service(path, r1)
        }
    }
}

fn serve_dir_layer(dir_list: &[DirRoute], initial: Router) -> Router {
    let serve_dir_items = dir_list
        .iter()
        .map(|dir_route| ServeDirItem {
            path: PathBuf::from(&dir_route.dir),
            base: dir_route.base.clone(),
        })
        .collect::<Vec<_>>();

    let services = serve_dir_items
        .iter()
        .map(|item| {
            let src = match &item.base {
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

    initial.layer(middleware::from_fn_with_state(services, try_many_serve_dir))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::handler_stack::HandlerStack::{Dirs, DirsProxy};
    use crate::server::router::common::to_resp_parts_and_body;
    use axum::body::Body;
    use bsnext_input::Input;
    use http::Request;
    use tower::ServiceExt;

    #[test]
    fn test_handler_stack_01() -> anyhow::Result<()> {
        let yaml = include_str!("../../../examples/basic/handler_stack.yml");
        let input = serde_yaml::from_str::<Input>(&yaml)?;

        {
            let first = input
                .servers
                .iter()
                .find(|x| x.identity.is_named("raw"))
                .unwrap();

            let expected = HandlerStack::Raw(RawRoute::Raw {
                raw: "another".to_string(),
            });

            let actual = routes_to_stack(&first.routes);

            assert_eq!(actual, expected);
        }

        {
            let first = input
                .servers
                .iter()
                .find(|x| x.identity.is_named("2dirs+proxy"))
                .unwrap();
            let expected = DirsProxy(
                vec![
                    DirRoute {
                        dir: "another".to_string(),
                        ..Default::default()
                    },
                    DirRoute {
                        dir: "another_2".to_string(),
                        ..Default::default()
                    },
                ],
                ProxyRoute {
                    proxy: "example.com".to_string(),
                },
            );

            let actual = routes_to_stack(&first.routes);

            assert_eq!(actual, expected);
        }

        {
            let first = input
                .servers
                .iter()
                .find(|s| s.identity.is_named("2dirs"))
                .unwrap();
            let expected = Dirs(vec![
                DirRoute {
                    dir: "public".to_string(),
                    ..Default::default()
                },
                DirRoute {
                    dir: ".".to_string(),
                    ..Default::default()
                },
            ]);

            let actual = routes_to_stack(&first.routes);

            assert_eq!(actual, expected);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_routes_to_router() -> anyhow::Result<()> {
        let yaml = include_str!("../../../examples/basic/handler_stack.yml");
        let input = serde_yaml::from_str::<Input>(&yaml)?;

        {
            let first = input
                .servers
                .iter()
                .find(|x| x.identity.is_named("raw"))
                .unwrap();

            let route_map = RouteMap::new_from_routes(&first.routes);
            let router = route_map.into_router();
            let request = Request::get("/styles.css").body(Body::empty())?;

            // Define the request
            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;
            let (_parts, body) = to_resp_parts_and_body(response).await;

            assert_eq!(body, "body { background: red }");
        }

        Ok(())
    }
}
