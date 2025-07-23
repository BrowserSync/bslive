use crate::handlers::proxy::{proxy_handler, ProxyConfig, RewriteKind};
use crate::optional_layers::optional_layers;
use crate::raw_loader::serve_raw_one;
use crate::route_candidate::RouteCandidate;
use crate::route_marker::RouteMarker;
use crate::route_match::RouteMatch;
use crate::runtime_ctx::RuntimeCtx;
use crate::server::router::ProxyResponseEncoding;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::handler::Handler;
use axum::middleware::from_fn;
use axum::response::IntoResponse;
use axum::routing::{any, any_service, get_service, MethodRouter};
use axum::{middleware, Extension, Router};
use bsnext_guards::{uri_extension, OuterUri};
use bsnext_input::route::{Route, RouteKind};
use bsnext_resp::{response_modifications_layer, InjectHandling};
use http::header::CONTENT_ENCODING;
use http::request::Parts;
use http::{StatusCode, Uri};
use std::collections::HashMap;
use std::path::PathBuf;
use tower::ServiceExt;
use tower_http::compression::{CompressionLayer, Predicate};
use tower_http::decompression::DecompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{debug, trace, trace_span};

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

    let path_and_query = outer_uri.path_and_query();

    trace!(?parts);
    debug!("will try {} candidate routes", routes.len());

    let candidates: Vec<RouteCandidate> = routes
        .iter()
        .enumerate()
        .filter(|(index, route)| {
            let span = trace_span!("early filter for candidates", index = index);
            let _g = span.enter();
            RouteMatch(route).route_match(&req, &outer_uri, &path, path_and_query, &parts)
        })
        .map(|(index, route)| RouteCandidate::for_route(index, route, &req, &uri, &outer_uri))
        .collect::<Vec<_>>();

    debug!("{} candidates passed early checks", candidates.len());

    let mut body: Option<Body> = Some(req.into_body());

    'find_candidates: for candidate in &candidates {
        let span = trace_span!("index", index = candidate.index);
        let _g = span.enter();

        trace!(?parts);

        let (next_body, control_flow) = candidate.try_exec(&mut body).await;
        if next_body.is_some() {
            body = next_body
        }

        if control_flow.is_break() {
            continue 'find_candidates;
        }

        trace!(mirror = ?candidate.mirror);

        let mut method_router = to_method_router(&path, &candidate.route.kind, &ctx);

        if let Some(ref injections) = candidate.injections {
            trace!(?injections);
            method_router = method_router.layer(from_fn(RouteMarker::before_proxy));
            method_router = method_router.layer(DecompressionLayer::new());
            method_router = method_router.layer(from_fn(RouteMarker::after_decompress));
            method_router = method_router.layer(middleware::from_fn(response_modifications_layer));
            method_router = method_router.layer(Extension(InjectHandling {
                items: injections.items().clone(),
            }));
            method_router =
                method_router.layer(CompressionLayer::new().compress_when(OnlyCompressWhen));
        }

        // decompress if needed
        // if let Some(mirror_path) = &candidate.mirror {
        //     method_router = method_router.layer(map_response(mapper));
        //     method_router = method_router.layer(DecompressionLayer::new());
        //     method_router = method_router.layer(middleware::from_fn_with_state(
        //         mirror_path.to_owned(),
        //         mirror_handler,
        //     ));
        //     method_router = com(method_router)
        // }

        let method_router: MethodRouter = optional_layers(method_router, &candidate.route.opts);
        let req_clone = match candidate.route.kind {
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

        // MAKE THE REQUEST
        let result = method_router.oneshot(req_clone).await;

        match result {
            Ok(result) => match result.status().as_u16() {
                // todo: this is way too simplistic, it should allow 404 being deliberately returned etc
                404 | 405 => {
                    if let Some(fallback) = &candidate.route.fallback {
                        let mr = to_method_router(&path, &fallback.kind, &ctx);
                        let raw_out: MethodRouter = optional_layers(mr, &fallback.opts);
                        let raw_fb = Request::from_parts(parts.clone(), Body::empty());
                        return raw_out.oneshot(raw_fb).await.into_response();
                    }
                }
                _ => {
                    trace!(
                        ?candidate.index,
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
#[derive(Debug, Clone)]
struct OnlyCompressWhen;
impl Predicate for OnlyCompressWhen {
    fn should_compress<B>(&self, response: &http::Response<B>) -> bool
    where
        B: http_body::Body,
    {
        match (
            response.extensions().get::<ProxyResponseEncoding>(),
            response.headers().get(CONTENT_ENCODING),
        ) {
            // if the original CONTENT_ENCODING header was removed
            (Some(..), None) => true,
            _ => false,
        }
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
