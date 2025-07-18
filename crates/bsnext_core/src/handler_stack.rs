use crate::handlers::proxy::{proxy_handler, ProxyConfig, RewriteKind};
use crate::optional_layers::optional_layers;
use crate::raw_loader::serve_raw_one;
use crate::runtime_ctx::RuntimeCtx;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::handler::Handler;
use axum::middleware::{from_fn, map_response, Next};
use axum::response::IntoResponse;
use axum::routing::{any, any_service, get_service, MethodRouter};
use axum::{middleware, Extension, Router};
use axum_extra::middleware::option_layer;
use bsnext_guards::route_guard::RouteGuard;
use bsnext_guards::{uri_extension, OuterUri};
use bsnext_input::route::{ListOrSingle, ProxyRoute, Route, RouteKind};
use bsnext_input::when_guard::{HasGuard, JsonGuard, JsonPropGuard, WhenBodyGuard, WhenGuard};
use bsnext_resp::InjectHandling;
use bytes::Bytes;
use http::header::{ACCEPT, CONTENT_TYPE};
use http::request::Parts;
use http::uri::PathAndQuery;
use http::{Method, Response, StatusCode, Uri};
use http_body_util::BodyExt;
use serde_json::Value;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::io;
use std::ops::ControlFlow;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::fs::{create_dir_all, File};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tower::ServiceExt;
use tower_http::decompression::DecompressionLayer;
use tower_http::services::{ServeDir, ServeFile};
use tracing::{debug, error, trace, trace_span};

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
    debug!("will try {} candidate routes", routes.len());

    let candidates: Vec<RouteCandidate> = routes
        .iter()
        .enumerate()
        .filter(|(index, route)| {
            let span = trace_span!("early filter for candidates", index = index);
            let _g = span.enter();

            trace!(?route.kind);

            // early checks from parts only
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

            // if this routes wants to inspect the body, check it was a POST
            let can_consume = match &route.when_body {
                None => true,
                Some(when_body) => {
                    let consuming = NeedsJsonGuard(when_body).accept_req(&req, &outer_uri);
                    trace!(route.when_body.json = consuming);
                    consuming
                }
            };

            trace!(?can_serve);
            trace!(?can_consume);

            if !can_serve || !can_consume {
                return false;
            }

            true
        })
        .map(|(index, route)| {
            let consume = route
                .when_body
                .as_ref()
                .is_some_and(|body| NeedsJsonGuard(body).accept_req(&req, &outer_uri));

            let css_req = req
                .headers()
                .get(ACCEPT)
                .and_then(|h| h.to_str().ok())
                .map(|c| c.contains("text/css"))
                .unwrap_or(false);

            let js_req = Path::new(req.uri().path())
                .extension()
                .is_some_and(|ext| ext == OsStr::new("js"));
            let mirror = if (css_req || js_req) {
                RouteHelper(&route).mirror().map(|v| v.to_path_buf())
            } else {
                None
            };

            let injections = route.opts.inject.as_injections();
            let req_accepted = injections
                .items
                .iter()
                .any(|item| item.accept_req(&req, &outer_uri));

            RouteCandidate {
                index,
                consume,
                route,
                mirror,
                inject: req_accepted,
            }
        })
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

        // decompress if needed
        if candidate.mirror.is_some() || candidate.inject {
            method_router = method_router.layer(DecompressionLayer::new());
        }

        if let Some(mirror_path) = &candidate.mirror {
            method_router = method_router.layer(middleware::from_fn_with_state(
                mirror_path.to_owned(),
                mirror_handler,
            ));
        }

        let raw_out: MethodRouter = optional_layers(method_router, &candidate.route.opts);
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
        let result = raw_out.oneshot(req_clone).await;

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

struct RouteHelper<'a>(pub &'a Route);

impl<'a> RouteHelper<'a> {
    fn mirror(&self) -> Option<&Path> {
        match &self.0.kind {
            RouteKind::Proxy(ProxyRoute {
                unstable_mirror, ..
            }) => unstable_mirror.as_ref().map(|s| Path::new(s)),
            _ => None,
        }
    }
}

#[derive(Debug)]
struct RouteCandidate<'a> {
    index: usize,
    route: &'a Route,
    consume: bool,
    mirror: Option<PathBuf>,
    inject: bool,
}

impl RouteCandidate<'_> {
    pub async fn try_exec(&self, body: &mut Option<Body>) -> (Option<Body>, ControlFlow<()>) {
        if self.consume {
            trace!("trying to collect body because candidate needs it");
            match_json_body(body, self.route).await
        } else {
            (None, ControlFlow::Continue(()))
        }
    }
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

async fn mirror_handler(
    State(path): State<PathBuf>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    let (mut sender, receiver) = tokio::sync::mpsc::channel::<Result<Bytes, io::Error>>(100);
    let as_stream = ReceiverStream::from(receiver);
    let c = req.uri().clone();
    let p = path.join(c.path().strip_prefix("/").unwrap());

    let r = next.run(req).await;
    let s = r.into_body().into_data_stream();

    tokio::spawn(async move {
        let s = s.throttle(Duration::from_millis(10));
        tokio::pin!(s);
        create_dir_all(&p.parent().unwrap()).await.unwrap();
        let mut file = BufWriter::new(File::create(p).await.unwrap());

        while let Some(Ok(b)) = s.next().await {
            match file.write(&b).await {
                Ok(_) => {}
                Err(e) => error!(?e, "could not write"),
            };
            // match file.write("\n".as_bytes()).await {
            //     Ok(_) => {}
            //     Err(e) => error!(?e, "could not new line"),
            // };
            match file.flush().await {
                Ok(_) => {}
                Err(e) => error!(?e, "could not flush"),
            };
            match sender.send(Ok(b)).await {
                Ok(_) => {}
                Err(e) => {
                    error!(?e, "sender was dropped before reading was finished");
                    error!("will break");
                    break;
                }
            };
        }
    });

    Body::from_stream(as_stream).into_response()
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

struct NeedsJsonGuard<'a>(pub &'a ListOrSingle<WhenBodyGuard>);
impl RouteGuard for NeedsJsonGuard<'_> {
    #[tracing::instrument(skip_all, name = "NeedsJsonGuard.accept_req")]
    fn accept_req(&self, req: &Request, _outer_uri: &Uri) -> bool {
        let exec = match self.0 {
            ListOrSingle::WhenOne(WhenBodyGuard::Json { .. }) => true,
            ListOrSingle::WhenMany(items) => items
                .iter()
                .any(|item| matches!(item, WhenBodyGuard::Json { .. })),
            _ => false,
        };
        trace!(?exec);
        if !exec {
            return false;
        }
        let headers = req.headers();
        let method = req.method();
        let json = headers.get(CONTENT_TYPE).is_some_and(|header| {
            header
                .to_str()
                .ok()
                .map(|h| h.contains("application/json"))
                .unwrap_or(false)
        });
        trace!(?json, ?method, ?headers);
        json && method == Method::POST
    }

    fn accept_res<T>(&self, _res: &Response<T>, _outer_uri: &Uri) -> bool {
        true
    }
}

impl NeedsJsonGuard<'_> {
    pub fn match_body(&self, value: &Value) -> bool {
        let matches: Vec<(&'_ WhenBodyGuard, bool)> = match self.0 {
            ListOrSingle::WhenOne(one) => vec![(one, match_one_json(value, one))],
            ListOrSingle::WhenMany(many) => many
                .iter()
                .map(|guard| (guard, match_one_json(value, guard)))
                .collect(),
        };
        matches.iter().all(|(_item, result)| *result)
    }
}

pub fn match_one_json(value: &Value, when_body_guard: &WhenBodyGuard) -> bool {
    match when_body_guard {
        WhenBodyGuard::Json { json } => match json {
            JsonGuard::ArrayLast { items, last } => match value.pointer(items) {
                Some(Value::Array(arr)) => match arr.last() {
                    None => false,
                    Some(last_item) => last.iter().all(|prop_guard| match prop_guard {
                        JsonPropGuard::PathIs { path, is } => match last_item.pointer(path) {
                            Some(Value::String(val_string)) => val_string == is,
                            _ => false,
                        },
                        JsonPropGuard::PathHas { path, has } => match last_item.pointer(path) {
                            Some(Value::String(val_string)) => val_string.contains(has),
                            _ => false,
                        },
                    }),
                },
                _ => false,
            },
            JsonGuard::ArrayAny { items, any } => match value.pointer(items) {
                Some(Value::Array(arr)) if arr.is_empty() => false,
                Some(Value::Array(val)) => val
                    .iter()
                    .any(|val| any.iter().any(|prop_guard| match_prop(val, prop_guard))),
                _ => false,
            },
            JsonGuard::ArrayAll { items, all } => match value.pointer(items) {
                Some(Value::Array(arr)) if arr.is_empty() => false,
                Some(Value::Array(arr)) => arr
                    .iter()
                    .any(|one_val| all.iter().all(|guard| match_prop(one_val, guard))),
                _ => false,
            },
            JsonGuard::Path(pg) => match_prop(value, pg),
        },
        WhenBodyGuard::Never => false,
    }
}

async fn match_json_body(
    body: &mut Option<Body>,
    route: &Route,
) -> (Option<Body>, ControlFlow<()>) {
    if let Some(inner_body) = body.take() {
        let collected = inner_body.collect();
        let bytes = match collected.await {
            Ok(collected) => collected.to_bytes(),
            Err(err) => {
                tracing::error!(?err, "could not collect bytes...");
                Bytes::new()
            }
        };

        trace!("did collect {} bytes", bytes.len());

        match serde_json::from_slice(bytes.iter().as_slice()) {
            Ok(value) => {
                let result = route
                    .when_body
                    .as_ref()
                    .map(|when_body| NeedsJsonGuard(when_body).match_body(&value));
                if result.is_some_and(|res| !res) {
                    trace!("ignoring, `when_body` was present, but didn't match the guards");
                    trace!("restoring body from clone");
                    (Some(Body::from(bytes)), ControlFlow::Break(()))
                } else {
                    if result.is_some() {
                        trace!("✅ when_body produced a valid match");
                    } else {
                        trace!("when_body didn't produce a value");
                    }
                    (Some(Body::from(bytes)), ControlFlow::Continue(()))
                }
            }
            Err(err) => {
                tracing::error!(?err, "could not deserialize into Value");
                (Some(Body::from(bytes)), ControlFlow::Continue(()))
            }
        }
    } else {
        trace!("could not .take() body");
        (None, ControlFlow::Continue(()))
    }
}

pub fn match_prop(value: &Value, prop_guard: &JsonPropGuard) -> bool {
    match prop_guard {
        JsonPropGuard::PathIs { path, is } => match value.pointer(path) {
            Some(Value::String(val_string)) => val_string == is,
            _ => false,
        },
        JsonPropGuard::PathHas { path, has } => match value.pointer(path) {
            Some(Value::String(val_string)) => val_string.contains(has),
            _ => false,
        },
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
