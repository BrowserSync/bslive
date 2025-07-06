use crate::handlers::proxy::{proxy_handler, ProxyConfig};
use crate::not_found::not_found_service::not_found_loader;
use crate::optional_layers::optional_layers;
use crate::raw_loader::serve_raw_one;
use crate::runtime_ctx::RuntimeCtx;
use crate::serve_dir::try_many_services_dir;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::handler::Handler;
use axum::middleware::{from_fn, from_fn_with_state, Next};
use axum::response::{IntoResponse, Response};
use axum::routing::{any, any_service, get_service, MethodRouter};
use axum::{middleware, Extension, Router};
use bsnext_guards::route_guard::RouteGuard;
use bsnext_input::route::{DirRoute, FallbackRoute, Opts, ProxyRoute, RawRoute, Route, RouteKind};
use bsnext_input::when_guard::{HasGuard, WhenGuard};
use http::request::Parts;
use http::{Method, StatusCode, Uri};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

#[derive(Debug, PartialEq)]
pub struct HandlerStackAlt {
    pub routes: Vec<Route>,
}

#[derive(Debug, PartialEq)]
pub enum HandlerStack {
    None,
    // todo: make this a separate thing
    Raw(RawRouteOpts),
    RawAndDirs {
        raw: RawRouteOpts,
        dirs: Vec<DirRouteOpts>,
    },
    Dirs(Vec<DirRouteOpts>),
    Proxy {
        proxy: ProxyRoute,
        opts: Opts,
    },
    DirsProxy {
        dirs: Vec<DirRouteOpts>,
        proxy: ProxyRoute,
        opts: Opts,
    },
}

#[derive(Debug, PartialEq)]
pub struct DirRouteOpts {
    dir_route: DirRoute,
    opts: Opts,
    fallback_route: Option<FallbackRoute>,
}

#[derive(Debug, PartialEq)]
pub struct RawRouteOpts {
    raw_route: RawRoute,
    opts: Opts,
}

impl DirRouteOpts {
    pub fn as_serve_dir(&self, cwd: &Path) -> ServeDir {
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
                let pb = PathBuf::from(&self.dir_route.dir);
                if pb.is_absolute() {
                    tracing::trace!("no root given, using `{}` directly", self.dir_route.dir);
                    ServeDir::new(&self.dir_route.dir)
                } else {
                    let joined = cwd.join(pb);
                    tracing::trace!(
                        "prepending the current directory to relative path {} {}",
                        cwd.display(),
                        joined.display()
                    );
                    ServeDir::new(joined)
                }
            }
        }
        .append_index_html_on_directories(true)
    }
    pub fn as_serve_file(&self) -> ServeFile {
        match &self.dir_route.base {
            Some(base_dir) => {
                tracing::trace!(
                    "combining root: `{}` with given path: `{}`",
                    base_dir.display(),
                    self.dir_route.dir
                );
                ServeFile::new(base_dir.join(&self.dir_route.dir))
            }
            None => {
                tracing::trace!("no root given, using `{}` directly", self.dir_route.dir);
                ServeFile::new(&self.dir_route.dir)
            }
        }
    }
}

impl DirRouteOpts {
    fn new(p0: DirRoute, p1: Opts, fallback_route: Option<FallbackRoute>) -> DirRouteOpts {
        Self {
            dir_route: p0,
            opts: p1,
            fallback_route,
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
                    acc.entry(route.path.as_str().to_string())
                        .and_modify(|acc| acc.push(route.clone()))
                        .or_insert(vec![route.clone()]);
                    acc
                }),
        }
    }

    pub fn into_router(self, ctx: &RuntimeCtx) -> Router {
        let mut router = Router::new();

        tracing::trace!("processing `{}` different routes", self.mapping.len());

        for (path, route_list) in self.mapping {
            tracing::trace!(
                "processing path: `{}` with `{}` routes",
                path,
                route_list.len()
            );

            // let stack = routes_to_stack(route_list);
            // let path_router = stack_to_router(&path, stack, ctx);
            let stack = routes_to_alt_stack(path.as_str(), route_list, ctx.clone());
            tracing::trace!("will merge router at path: `{path}`");

            tracing::trace!("will merge router at path: `{path}`");
            router = router.merge(stack);
        }

        router
    }

    pub fn into_alt_router(self, ctx: &RuntimeCtx) -> Router {
        let mut router = Router::new();

        tracing::trace!("processing `{}` different routes", self.mapping.len());

        for (path, route_list) in self.mapping {
            tracing::trace!(
                "processing path: `{}` with `{}` routes",
                path,
                route_list.len()
            );

            let stack = routes_to_alt_stack(path.as_str(), route_list, ctx.clone());
            tracing::trace!("will merge router at path: `{path}`");

            router = router.merge(stack);
        }

        router
    }
}

pub fn append_stack(state: HandlerStack, route: Route) -> HandlerStack {
    match state {
        HandlerStack::None => match route.kind {
            RouteKind::Raw(raw_route) => HandlerStack::Raw(RawRouteOpts {
                raw_route,
                opts: route.opts,
            }),
            RouteKind::Proxy(new_proxy_route) => HandlerStack::Proxy {
                proxy: new_proxy_route,
                opts: route.opts,
            },
            RouteKind::Dir(dir) => {
                HandlerStack::Dirs(vec![DirRouteOpts::new(dir, route.opts, route.fallback)])
            }
        },
        HandlerStack::Raw(RawRouteOpts { raw_route, opts }) => match route.kind {
            // if a second 'raw' is seen, just use it, discarding the previous
            RouteKind::Raw(raw_route) => HandlerStack::Raw(RawRouteOpts {
                raw_route,
                opts: route.opts,
            }),
            RouteKind::Dir(dir) => HandlerStack::RawAndDirs {
                dirs: vec![DirRouteOpts::new(dir, route.opts, None)],
                raw: RawRouteOpts { raw_route, opts },
            },
            // 'raw' handlers never get updated
            _ => HandlerStack::Raw(RawRouteOpts { raw_route, opts }),
        },
        HandlerStack::RawAndDirs { .. } => {
            todo!("support RawAndDirs")
        }
        HandlerStack::Dirs(mut dirs) => match route.kind {
            RouteKind::Dir(next_dir) => {
                dirs.push(DirRouteOpts::new(next_dir, route.opts, route.fallback));
                HandlerStack::Dirs(dirs)
            }
            RouteKind::Proxy(proxy) => HandlerStack::DirsProxy {
                dirs,
                proxy,
                opts: route.opts,
            },
            _ => HandlerStack::Dirs(dirs),
        },
        HandlerStack::Proxy { proxy, opts } => match route.kind {
            RouteKind::Dir(dir) => HandlerStack::DirsProxy {
                dirs: vec![DirRouteOpts::new(dir, route.opts, route.fallback)],
                proxy,
                opts,
            },
            _ => HandlerStack::Proxy { proxy, opts },
        },
        HandlerStack::DirsProxy {
            mut dirs,
            proxy,
            opts,
        } => match route.kind {
            RouteKind::Dir(dir) => {
                dirs.push(DirRouteOpts::new(dir, route.opts, route.fallback));
                HandlerStack::DirsProxy { dirs, proxy, opts }
            }
            // todo(alpha): how to handle multiple proxies? should it just override for now?
            _ => HandlerStack::DirsProxy { dirs, proxy, opts },
        },
    }
}

pub fn fallback_to_layered_method_router(route: FallbackRoute) -> MethodRouter {
    match route.kind {
        RouteKind::Raw(raw_route) => {
            let svc = any_service(serve_raw_one.with_state(raw_route));
            optional_layers(svc, &route.opts)
        }
        RouteKind::Proxy(_new_proxy_route) => {
            // todo(alpha): make a decision proxy as a fallback
            todo!("add support for RouteKind::Proxy as a fallback?")
        }
        RouteKind::Dir(dir) => {
            tracing::trace!("creating fallback for dir {:?}", dir);
            let item = DirRouteOpts::new(dir, route.opts, None);
            let serve_dir_service = item.as_serve_file();
            let service = get_service(serve_dir_service);
            optional_layers(service, &item.opts)
        }
    }
}

pub fn routes_to_stack(routes: Vec<Route>) -> HandlerStack {
    routes.into_iter().fold(HandlerStack::None, append_stack)
}

pub fn routes_to_alt_stack(path: &str, routes: Vec<Route>, ctx: RuntimeCtx) -> Router {
    let r = Router::new().layer(from_fn_with_state((path.to_string(), routes, ctx), try_one));
    Router::new().nest_service(path, r)
}

pub async fn try_one(
    State((path, routes, ctx)): State<(String, Vec<Route>, RuntimeCtx)>,
    uri: Uri,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    println!("before uri={uri}, path={path}");

    let (original_parts, original_body) = req.into_parts();

    for (index, route) in routes.into_iter().enumerate() {
        let accept = route.when.as_ref().unwrap_or(&WhenGuard::Always);

        let will_serve = match accept {
            WhenGuard::Always => true,
            WhenGuard::Never => false,
            WhenGuard::Query { query } => query
                .iter()
                .map(QueryHasGuard)
                .any(|x| x.accept_req_parts(&original_parts)),
        };

        if !will_serve {
            continue;
        }

        let m_r = match route.kind {
            RouteKind::Raw(raw) => any_service(serve_raw_one.with_state(raw.clone())),
            RouteKind::Proxy(_) => todo!("RouteKind::Proxy"),
            RouteKind::Dir(dir_route) => {
                let svc = match &dir_route.base {
                    Some(base_dir) => {
                        tracing::trace!(
                            "combining root: `{}` with given path: `{}`",
                            base_dir.display(),
                            dir_route.dir
                        );
                        ServeDir::new(base_dir.join(&dir_route.dir))
                    }
                    None => {
                        let pb = PathBuf::from(&dir_route.dir);
                        if pb.is_absolute() {
                            tracing::trace!("no root given, using `{}` directly", dir_route.dir);
                            ServeDir::new(&dir_route.dir)
                        } else {
                            let joined = ctx.cwd().join(pb);
                            tracing::trace!(
                                "prepending the current directory to relative path {} {}",
                                ctx.cwd().display(),
                                joined.display()
                            );
                            ServeDir::new(joined)
                        }
                    }
                }
                .append_index_html_on_directories(true);
                get_service(svc)
            }
        };

        let raw_out = optional_layers(m_r, &route.opts);
        let req_clone = Request::from_parts(original_parts.clone(), Body::empty());
        let result = raw_out.oneshot(req_clone).await;

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
                println!("got a result {index}");
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

    let r = Request::from_parts(original_parts.clone(), original_body);
    let res = next.run(r).await;
    println!("after");
    res
}

pub async fn serve_one(uri: Uri, req: Request) -> Response {
    (StatusCode::NOT_FOUND, format!("No route for {uri}")).into_response()
}

struct QueryHasGuard<'a>(pub &'a HasGuard);

impl RouteGuard for QueryHasGuard<'_> {
    fn accept_req_parts(&self, parts: &Parts) -> bool {
        let Some(query) = parts.uri.query() else {
            return false;
        };
        match &self.0 {
            HasGuard::Is { is } => is == query,
            HasGuard::Has { has } => query.contains(has),
            HasGuard::NotHas { not_has } => !query.contains(not_has),
        }
    }
}

pub fn stack_to_router(path: &str, stack: HandlerStack, ctx: &RuntimeCtx) -> Router {
    match stack {
        HandlerStack::None => unreachable!(),
        HandlerStack::Raw(RawRouteOpts { raw_route, opts }) => {
            let svc = any_service(serve_raw_one.with_state(raw_route));
            let out = optional_layers(svc, &opts);
            Router::new().route_service(path, out)
        }
        HandlerStack::RawAndDirs {
            dirs,
            raw: RawRouteOpts { raw_route, opts },
        } => {
            let svc = any_service(serve_raw_one.with_state(raw_route));
            let raw_out = optional_layers(svc, &opts);
            let service = serve_dir_layer(&dirs, Router::new(), ctx);
            Router::new().route(path, raw_out).fallback_service(service)
        }
        HandlerStack::Dirs(dirs) => {
            let service = serve_dir_layer(&dirs, Router::new(), ctx);
            Router::new()
                .nest_service(path, service)
                .layer(from_fn(not_found_loader))
        }
        HandlerStack::Proxy { proxy, opts } => {
            let proxy_config = ProxyConfig {
                target: proxy.proxy.clone(),
                path: path.to_string(),
            };

            let proxy_with_decompression = proxy_handler.layer(Extension(proxy_config.clone()));
            let as_service = any(proxy_with_decompression);

            Router::new().nest_service(path, optional_layers(as_service, &opts))
        }
        HandlerStack::DirsProxy { dirs, proxy, opts } => {
            let proxy_router = stack_to_router(path, HandlerStack::Proxy { proxy, opts }, ctx);
            let r1 = serve_dir_layer(&dirs, Router::new().fallback_service(proxy_router), ctx);
            Router::new().nest_service(path, r1)
        }
    }
}

fn serve_dir_layer(
    dir_list_with_opts: &[DirRouteOpts],
    initial: Router,
    ctx: &RuntimeCtx,
) -> Router {
    let serve_dir_items = dir_list_with_opts
        .iter()
        .map(|dir_route| match &dir_route.fallback_route {
            None => {
                let serve_dir_service = dir_route.as_serve_dir(ctx.cwd());
                let service = get_service(serve_dir_service);
                optional_layers(service, &dir_route.opts)
            }
            Some(fallback) => {
                let stack = fallback_to_layered_method_router(fallback.clone());
                let serve_dir_service = dir_route
                    .as_serve_dir(ctx.cwd())
                    .fallback(stack)
                    .call_fallback_on_method_not_allowed(true);
                let service = any_service(serve_dir_service);
                optional_layers(service, &dir_route.opts)
            }
        })
        .collect::<Vec<MethodRouter>>();

    initial.layer(from_fn_with_state(serve_dir_items, try_many_services_dir))
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::server::router::common::to_resp_parts_and_body;
    use axum::body::Body;
    use std::env::current_dir;

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
            .map(ToOwned::to_owned)
            .unwrap();

        let actual = routes_to_stack(first.routes);
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
            .map(ToOwned::to_owned)
            .unwrap();

        let actual = routes_to_stack(first.routes);

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
            .map(ToOwned::to_owned)
            .unwrap();

        let actual = routes_to_stack(first.routes);

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
            let router = route_map.into_alt_router(&RuntimeCtx::default());
            let request = Request::get("/styles.css").body(Body::empty())?;

            // Define the request
            // Make a one-shot request on the router
            let response = router.oneshot(request).await?;
            let (parts, body) = to_resp_parts_and_body(response).await;

            assert_eq!(parts.status.as_u16(), 200);
            assert_eq!(body, "body { background: red }");
        }

        Ok(())
    }
    #[tokio::test]
    async fn test_raw_with_dir_fallback() -> anyhow::Result<()> {
        let yaml = include_str!("../../../examples/basic/handler_stack.yml");
        let input = serde_yaml::from_str::<Input>(&yaml)?;

        {
            let first = input
                .servers
                .iter()
                .find(|x| x.identity.is_named("raw+dir"))
                .unwrap();

            let route_map = RouteMap::new_from_routes(&first.routes);
            let router = route_map.into_alt_router(&RuntimeCtx::default());
            let raw_request = Request::get("/").body(Body::empty())?;
            let response = router.oneshot(raw_request).await?;
            let (_parts, body) = to_resp_parts_and_body(response).await;
            assert_eq!(body, "hello world!");

            let cwd = current_dir().unwrap();
            let cwd = cwd.ancestors().nth(2).unwrap();
            let ctx = RuntimeCtx::new(cwd);
            let route_map = RouteMap::new_from_routes(&first.routes);
            let router = route_map.into_alt_router(&ctx);
            let dir_request = Request::get("/script.js").body(Body::empty())?;
            let response = router.oneshot(dir_request).await?;
            let (_parts, body) = to_resp_parts_and_body(response).await;
            let expected = include_str!("../../../examples/basic/public/script.js");
            assert_eq!(body, expected);
        }

        Ok(())
    }
}
