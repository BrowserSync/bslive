use crate::handlers::proxy::{proxy_handler, ProxyConfig};
use crate::optional_layers::{optional_layers, optional_layers_lol};
use crate::raw_loader::serve_raw_one;
use crate::runtime_ctx::RuntimeCtx;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::handler::Handler;
use axum::middleware::{from_fn, from_fn_with_state, Next};
use axum::response::IntoResponse;
use axum::routing::{any, any_service, get_service, MethodRouter};
use axum::{middleware, Extension, Router};
use bsnext_guards::route_guard::RouteGuard;
use bsnext_input::route::{Route, RouteKind};
use bsnext_input::when_guard::{HasGuard, WhenGuard};
use http::request::Parts;
use http::{Method, StatusCode, Uri};
use std::collections::HashMap;
use std::path::PathBuf;
use tower::ServiceExt;
use tower_http::services::ServeDir;
use tracing::{trace, trace_span};
// impl DirRouteOpts {
//     pub fn as_serve_file(&self) -> ServeFile {
//         match &self.dir_route.base {
//             Some(base_dir) => {
//                 tracing::trace!(
//                     "combining root: `{}` with given path: `{}`",
//                     base_dir.display(),
//                     self.dir_route.dir
//                 );
//                 ServeFile::new(base_dir.join(&self.dir_route.dir))
//             }
//             None => {
//                 tracing::trace!("no root given, using `{}` directly", self.dir_route.dir);
//                 ServeFile::new(&self.dir_route.dir)
//             }
//         }
//     }
// }

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

    #[tracing::instrument(skip(self))]
    pub fn into_router(self, ctx: &RuntimeCtx) -> Router {
        let mut router = Router::new();

        trace!("processing `{}` different routes", self.mapping.len());

        for (index, (path, route_list)) in self.mapping.into_iter().enumerate() {
            trace!(?index, ?path, "creating for `{}` routes", route_list.len());

            // let stack = routes_to_stack(route_list);
            // let path_router = stack_to_router(&path, stack, ctx);
            let stack = route_list_for_path(path.as_str(), route_list, ctx.clone());

            trace!(?index, ?path, "will merge router");
            router = router.merge(stack);
        }

        router
    }
}

#[tracing::instrument(skip_all)]
pub fn route_list_for_path(path: &str, routes: Vec<Route>, ctx: RuntimeCtx) -> Router {
    // let r1 = from_fn_with_state((path.to_string(), routes, ctx), try_one);
    let svc = any_service(try_one.with_state((path.to_string(), routes, ctx)));
    // if path.contains("{") {
    //     tracing::trace!(?path, "route");
    //     return Router::new().route(path, svc);
    // }
    tracing::trace!("nest_service");
    Router::new()
        .nest_service(path, svc)
        .layer(from_fn(uri_extension))
}
#[derive(Debug, Clone)]
pub struct OuterUri(pub Uri);
pub async fn uri_extension(uri: Uri, mut req: Request, next: Next) -> impl IntoResponse {
    req.extensions_mut().insert(OuterUri(uri));
    next.run(req).await
}

pub async fn try_one(
    State((path, routes, ctx)): State<(String, Vec<Route>, RuntimeCtx)>,
    Extension(uri): Extension<OuterUri>,
    local_uri: Uri,
    req: Request,
) -> impl IntoResponse {
    let span = trace_span!("try_one", uri = ?uri.0, path = path, local_uri = ?local_uri);
    let _g = span.enter();

    let (mut original_parts, original_body) = req.into_parts();
    // original_parts.uri = uri.0;
    trace!(?original_parts);
    trace!("will try {} routes", routes.len());

    for (index, route) in routes.into_iter().enumerate() {
        let span = trace_span!("index", index = index);
        let _g = span.enter();

        let accept = route.when.as_ref().unwrap_or(&WhenGuard::Always);
        trace!(?accept);

        let can_serve = match accept {
            WhenGuard::Always => true,
            WhenGuard::Never => false,
            WhenGuard::Query { query } => query
                .iter()
                .map(QueryHasGuard)
                .any(|x| x.accept_req_parts(&original_parts)),
        };

        trace!(?can_serve);

        if !can_serve {
            continue;
        }

        trace!(?original_parts);

        let result = match to_method_router(&path, &route, &ctx) {
            Either::Left(router) => {
                let raw_out = optional_layers(router, &route.opts);
                let req_clone = Request::from_parts(original_parts.clone(), Body::empty());
                raw_out.oneshot(req_clone).await
            }
            Either::Right(method_router) => {
                let raw_out = optional_layers_lol(method_router, &route.opts);
                let req_clone = Request::from_parts(original_parts.clone(), Body::empty());
                raw_out.oneshot(req_clone).await
            }
        };

        match result {
            Ok(result) if result.status() == 404 => {
                trace!("  ❌ not found at index {}, trying another", index);
                continue;
            }
            Ok(result) if result.status() == 405 => {
                trace!("  ❌ 405, trying another...");
                continue;
            }
            Ok(result) => {
                println!("got a result {index}");
                trace!(
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

    StatusCode::NOT_FOUND.into_response()
}

enum Either {
    Left(Router),
    Right(MethodRouter),
}
fn to_method_router(path: &str, route: &Route, ctx: &RuntimeCtx) -> Either {
    match &route.kind {
        RouteKind::Raw(raw) => Either::Right(any_service(serve_raw_one.with_state(raw.clone()))),
        RouteKind::Proxy(proxy) => {
            let proxy_config = ProxyConfig {
                target: proxy.proxy.clone(),
                path: path.to_string(),
            };
            let proxy_with_decompression = proxy_handler.layer(Extension(proxy_config.clone()));
            let as_service = any(proxy_with_decompression);
            Either::Left(Router::new().nest_service(path, as_service))
        }
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
                        trace!("no root given, using `{}` directly", dir_route.dir);
                        ServeDir::new(&dir_route.dir)
                    } else {
                        let joined = ctx.cwd().join(pb);
                        trace!(?joined, "serving");
                        ServeDir::new(joined)
                    }
                }
            }
            .append_index_html_on_directories(true);
            Either::Right(get_service(svc))
        }
    }
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::server::router::common::to_resp_parts_and_body;
    use axum::body::Body;
    use std::env::current_dir;

    use bsnext_input::Input;
    use http::Request;
    use tower::ServiceExt;

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
            let router = route_map.into_router(&RuntimeCtx::default());
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
            let router = route_map.into_router(&RuntimeCtx::default());
            let raw_request = Request::get("/").body(Body::empty())?;
            let response = router.oneshot(raw_request).await?;
            let (_parts, body) = to_resp_parts_and_body(response).await;
            assert_eq!(body, "hello world!");

            let cwd = current_dir().unwrap();
            let cwd = cwd.ancestors().nth(2).unwrap();
            let ctx = RuntimeCtx::new(cwd);
            let route_map = RouteMap::new_from_routes(&first.routes);
            let router = route_map.into_router(&ctx);
            let dir_request = Request::get("/script.js").body(Body::empty())?;
            let response = router.oneshot(dir_request).await?;
            let (_parts, body) = to_resp_parts_and_body(response).await;
            let expected = include_str!("../../../examples/basic/public/script.js");
            assert_eq!(body, expected);
        }

        Ok(())
    }
}
