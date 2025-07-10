use crate::handlers::proxy::{proxy_handler, ProxyConfig, RewriteKind};
use crate::optional_layers::optional_layers;
use crate::raw_loader::serve_raw_one;
use crate::runtime_ctx::RuntimeCtx;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::handler::Handler;
use axum::middleware::from_fn;
use axum::response::IntoResponse;
use axum::routing::{any, any_service, get_service, MethodRouter};
use axum::{Extension, Router};
use bsnext_guards::route_guard::RouteGuard;
use bsnext_guards::{uri_extension, OuterUri};
use bsnext_input::route::{ListOrSingle, Route, RouteKind};
use bsnext_input::when_guard::{HasGuard, WhenGuard};
use http::request::Parts;
use http::uri::PathAndQuery;
use http::{Response, StatusCode, Uri};
use std::collections::HashMap;
use std::path::PathBuf;
use tower::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{trace, trace_span};

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
    tracing::trace!("nest_service");
    Router::new()
        .nest_service(path, svc)
        .layer(from_fn(uri_extension))
}

pub async fn try_one(
    State((path, routes, ctx)): State<(String, Vec<Route>, RuntimeCtx)>,
    Extension(OuterUri(outer_uri)): Extension<OuterUri>,
    parts: Parts,
    uri: Uri,
    req: Request,
) -> impl IntoResponse {
    let span = trace_span!("try_one", outer_uri = ?outer_uri, path = path, local_uri = ?uri);
    let _g = span.enter();

    let pq = outer_uri.path_and_query();

    trace!(?parts);
    trace!("will try {} routes", routes.len());

    let candidates = routes
        .iter()
        .enumerate()
        .filter(|(index, route)| {
            let span = trace_span!("early filter for candidates", index = index);
            let _g = span.enter();

            trace!(?route.kind);

            let can_serve: bool = route
                .when
                .as_ref()
                .map(|when| match &when {
                    ListOrSingle::WhenOne(when) => match_one(when, &outer_uri, &path, pq, &parts),
                    ListOrSingle::WhenMany(many) => many
                        .iter()
                        .all(|when| match_one(when, &outer_uri, &path, pq, &parts)),
                })
                .unwrap_or(true);

            trace!(?can_serve);

            if !can_serve {
                return false;
            }

            true
        })
        .collect::<Vec<_>>();

    trace!("{} candidates", candidates.len());

    let mut body: Option<Body> = candidates.last().and_then(|(_, route)| {
        if matches!(route.kind, RouteKind::Proxy(..)) {
            trace!("will consume body for proxy");
            Some(req.into_body())
        } else {
            None
        }
    });

    for (index, route) in &candidates {
        let span = trace_span!("index", index = index);
        let _g = span.enter();

        trace!(?parts);

        let method_router = to_method_router(&path, &route.kind, &ctx);
        let raw_out: MethodRouter = optional_layers(method_router, &route.opts);
        let req_clone = match route.kind {
            RouteKind::Raw(_) => Request::from_parts(parts.clone(), Body::empty()),
            RouteKind::Proxy(_) => {
                if let Some(body) = body.take() {
                    Request::from_parts(parts.clone(), body)
                } else {
                    Request::from_parts(parts.clone(), Body::empty())
                }
            }
            RouteKind::Dir(_) => Request::from_parts(parts.clone(), Body::empty()),
        };

        let result = raw_out.oneshot(req_clone).await;

        match result {
            Ok(result) => match result.status().as_u16() {
                404 | 405 => {
                    if let Some(fallback) = &route.fallback {
                        let mr = to_method_router(&path, &fallback.kind, &ctx);
                        let raw_out: MethodRouter = optional_layers(mr, &fallback.opts);
                        let raw_fb = Request::from_parts(parts.clone(), Body::empty());
                        return raw_out.oneshot(raw_fb).await.into_response();
                    }
                }
                _ => {
                    trace!(
                        ?index,
                        " - ✅ a non-404 response was given {}",
                        result.status()
                    );
                    return result.into_response();
                }
            },
            Err(e) => {
                tracing::error!(?e);
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }

    tracing::trace!("StatusCode::NOT_FOUND");
    StatusCode::NOT_FOUND.into_response()
}

fn match_one(
    when_guard: &WhenGuard,
    outer_uri: &Uri,
    path: &str,
    pq: Option<&PathAndQuery>,
    parts: &Parts,
) -> bool {
    match when_guard {
        WhenGuard::Always => true,
        WhenGuard::Never => false,
        WhenGuard::ExactUri { exact_uri: true } => path == pq.map(|pq| pq.as_str()).unwrap_or("/"),
        WhenGuard::ExactUri { exact_uri: false } => path != pq.map(|pq| pq.as_str()).unwrap_or("/"),
        WhenGuard::Query { query } => QueryHasGuard(query).accept_req_parts(parts, outer_uri),
        WhenGuard::Accept { accept } => AcceptHasGuard(accept).accept_req_parts(parts, outer_uri),
    }
}

fn to_method_router(path: &str, route_kind: &RouteKind, ctx: &RuntimeCtx) -> MethodRouter {
    match route_kind {
        RouteKind::Raw(raw) => any_service(serve_raw_one.with_state(raw.clone())),
        RouteKind::Proxy(proxy) => {
            let proxy_config = ProxyConfig {
                target: proxy.proxy.clone(),
                path: path.to_string(),
                headers: proxy.proxy_headers.clone().unwrap_or_default(),
                rewrite_kind: RewriteKind::from(proxy.rewrite_uri),
            };
            let proxy_with_decompression = proxy_handler.layer(Extension(proxy_config.clone()));
            any(proxy_with_decompression)
        }
        RouteKind::Dir(dir_route) => {
            tracing::trace!(?dir_route);
            match &dir_route.base {
                Some(base_dir) => {
                    tracing::trace!(
                        "combining root: `{}` with given path: `{}`",
                        base_dir.display(),
                        dir_route.dir
                    );
                    get_service(ServeDir::new(base_dir.join(&dir_route.dir)))
                }
                None => {
                    let pb = PathBuf::from(&dir_route.dir);
                    if pb.is_file() {
                        get_service(ServeFile::new(pb))
                    } else if pb.is_absolute() {
                        trace!("no root given, using `{}` directly", dir_route.dir);
                        get_service(
                            ServeDir::new(&dir_route.dir).append_index_html_on_directories(true),
                        )
                    } else {
                        let joined = ctx.cwd().join(pb);
                        trace!(?joined, "serving");
                        get_service(ServeDir::new(joined).append_index_html_on_directories(true))
                    }
                }
            }
        }
    }
}

struct QueryHasGuard<'a>(pub &'a HasGuard);

impl RouteGuard for QueryHasGuard<'_> {
    fn accept_req(&self, _req: &Request, _outer_uri: &Uri) -> bool {
        true
    }

    fn accept_res<T>(&self, _res: &Response<T>, _outer_uri: &Uri) -> bool {
        true
    }

    fn accept_req_parts(&self, parts: &Parts, _outer_uri: &Uri) -> bool {
        let Some(query) = parts.uri.query() else {
            return false;
        };
        match &self.0 {
            HasGuard::Is { is } | HasGuard::Literal(is) => is == query,
            HasGuard::Has { has } => query.contains(has),
            HasGuard::NotHas { not_has } => !query.contains(not_has),
        }
    }
}
struct AcceptHasGuard<'a>(pub &'a HasGuard);

impl RouteGuard for AcceptHasGuard<'_> {
    fn accept_req(&self, _req: &Request, _outer_uri: &Uri) -> bool {
        true
    }

    fn accept_res<T>(&self, _res: &Response<T>, _outer_uri: &Uri) -> bool {
        true
    }

    fn accept_req_parts(&self, parts: &Parts, _outer_uri: &Uri) -> bool {
        let Some(query) = parts.headers.get("accept") else {
            return false;
        };
        let Ok(str) = std::str::from_utf8(query.as_bytes()) else {
            tracing::error!("bytes incorrrect");
            return false;
        };
        match &self.0 {
            HasGuard::Literal(is) | HasGuard::Is { is } => is == str,
            HasGuard::Has { has } => str.contains(has),
            HasGuard::NotHas { not_has } => !str.contains(not_has),
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
