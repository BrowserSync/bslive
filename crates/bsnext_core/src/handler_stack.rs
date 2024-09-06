use crate::common_layers::add_route_layers;
use crate::handlers::proxy::{proxy_handler, ProxyConfig};
use crate::raw_loader::serve_raw_one;
use crate::serve_dir::try_many_services_dir;
use axum::handler::Handler;
use axum::middleware::from_fn_with_state;
use axum::routing::{any, any_service, get_service, MethodRouter};
use axum::{Extension, Router};
use bsnext_input::route::{DirRoute, Opts, ProxyRoute, RawRoute, Route, RouteKind};
use bsnext_resp::path_matcher::PathMatcher::Def;
use std::collections::HashMap;
use tower_http::services::ServeDir;

#[derive(Debug, PartialEq)]
pub enum HandlerStack {
    None,
    // todo: make this a separate thing
    Raw { raw: RawRoute, opts: Opts },
    Dirs(Vec<DirRouteOpts>),
    Proxy { proxy: ProxyRoute, opts: Opts },
    DirsProxy(Vec<DirRouteOpts>, ProxyRoute),
}

#[derive(Debug, PartialEq)]
pub struct DirRouteOpts {
    dir_route: DirRoute,
    opts: Opts,
}

impl DirRouteOpts {
    pub fn as_serve_dir(&self) -> ServeDir {
        match &self.dir_route.base {
            Some(base_dir) => {
                tracing::trace!(
                    "combining root: `{}` with given path: `{}`",
                    base_dir.display(),
                    self.dir_route.dir
                );
                ServeDir::new(base_dir.join(&self.dir_route.dir))
            }
            None => {
                tracing::trace!("no root given, using `{}` directly", self.dir_route.dir);
                ServeDir::new(&self.dir_route.dir)
            }
        }
    }
}

impl DirRouteOpts {
    fn new(p0: DirRoute, p1: Opts) -> DirRouteOpts {
        Self {
            dir_route: p0,
            opts: p1,
        }
    }
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
            RouteKind::Raw(raw_route) => HandlerStack::Raw {
                raw: raw_route,
                opts: route.opts,
            },
            RouteKind::Proxy(new_proxy_route) => HandlerStack::Proxy {
                proxy: new_proxy_route,
                opts: route.opts,
            },
            RouteKind::Dir(dir) => HandlerStack::Dirs(vec![DirRouteOpts::new(dir, route.opts)]),
        },
        HandlerStack::Raw { raw, opts } => match route.kind {
            // if a second 'raw' is seen, just use it, discarding the previous
            RouteKind::Raw(raw_route) => HandlerStack::Raw {
                raw: raw_route,
                opts: route.opts,
            },
            // 'raw' handlers never get updated
            _ => HandlerStack::Raw { raw, opts },
        },
        HandlerStack::Dirs(mut dirs) => match route.kind {
            RouteKind::Dir(next_dir) => {
                dirs.push(DirRouteOpts::new(next_dir, route.opts));
                HandlerStack::Dirs(dirs)
            }
            RouteKind::Proxy(proxy) => HandlerStack::DirsProxy(dirs, proxy),
            _ => HandlerStack::Dirs(dirs),
        },
        HandlerStack::Proxy { proxy, opts } => match route.kind {
            RouteKind::Dir(dir) => {
                HandlerStack::DirsProxy(vec![DirRouteOpts::new(dir, route.opts)], proxy)
            }
            _ => HandlerStack::Proxy { proxy, opts },
        },
        HandlerStack::DirsProxy(mut dirs, proxy) => match route.kind {
            RouteKind::Dir(dir) => {
                dirs.push(DirRouteOpts::new(dir, route.opts));
                HandlerStack::DirsProxy(dirs, proxy)
            }
            // todo(alpha): how to handle multiple proxies? should it just override for now?
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
        HandlerStack::Raw { raw, opts } => {
            let svc = any_service(serve_raw_one.with_state(raw));
            Router::new().route_service(path, add_route_layers(svc, path, &opts))
        }
        HandlerStack::Dirs(dirs) => {
            Router::new().nest_service(path, serve_dir_layer(path, &dirs, Router::new()))
        }
        HandlerStack::Proxy { proxy, opts } => {
            let proxy_config = ProxyConfig {
                target: proxy.proxy.clone(),
                path: path.to_string(),
            };
            let router = any(proxy_handler).layer(Extension(proxy_config.clone()));
            Router::new().nest_service(path, add_route_layers(router, path, &opts))
        }
        HandlerStack::DirsProxy(dir_list, proxy) => {
            let r2 = stack_to_router(
                path,
                HandlerStack::Proxy {
                    proxy,
                    opts: Default::default(),
                },
            );
            let r1 = serve_dir_layer(path, &dir_list, Router::new().fallback_service(r2));
            Router::new().nest_service(path, r1)
        }
    }
}

fn serve_dir_layer(path: &str, dir_list_with_opts: &[DirRouteOpts], initial: Router) -> Router {
    let serve_dir_items = dir_list_with_opts
        .iter()
        .map(|dir_route| {
            let src = dir_route.as_serve_dir();
            let service = get_service(src);
            add_route_layers(service, path, &dir_route.opts)
        })
        .collect::<Vec<MethodRouter>>();

    initial.layer(from_fn_with_state(
        (path.to_string(), serve_dir_items),
        try_many_services_dir,
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::server::router::common::to_resp_parts_and_body;
    use axum::body::Body;
    use bsnext_input::Input;
    use http::Request;
    use insta::assert_debug_snapshot;
    use tower::ServiceExt;

    #[test]
    fn test_handler_stack_01() -> anyhow::Result<()> {
        let yaml = include_str!("../../../examples/basic/handler_stack.yml");
        let input = serde_yaml::from_str::<Input>(&yaml)?;
        let first = input
            .servers
            .iter()
            .find(|x| x.identity.is_named("raw"))
            .unwrap();

        let actual = routes_to_stack(&first.routes);
        assert_debug_snapshot!(actual);
        Ok(())
    }

    #[test]
    fn test_handler_stack_02() -> anyhow::Result<()> {
        let yaml = include_str!("../../../examples/basic/handler_stack.yml");
        let input = serde_yaml::from_str::<Input>(&yaml)?;
        let first = input
            .servers
            .iter()
            .find(|x| x.identity.is_named("2dirs+proxy"))
            .unwrap();

        let actual = routes_to_stack(&first.routes);

        assert_debug_snapshot!(actual);
        Ok(())
    }
    #[test]
    fn test_handler_stack_03() -> anyhow::Result<()> {
        let yaml = include_str!("../../../examples/basic/handler_stack.yml");
        let input = serde_yaml::from_str::<Input>(&yaml)?;
        let first = input
            .servers
            .iter()
            .find(|s| s.identity.is_named("raw+opts"))
            .unwrap();

        let actual = routes_to_stack(&first.routes);

        assert_debug_snapshot!(actual);
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
