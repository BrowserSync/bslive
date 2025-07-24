use crate::body_match::BodyMatch;
use crate::route_cache::CachePrevent;
use crate::route_compress::Compress;
use crate::route_cors::Cors;
use crate::route_delay::Delay;
use crate::route_effect::RouteEffect;
use crate::route_injections::Injections;
use crate::route_mirror::Mirror;
use crate::route_res_headers::ResHeaders;
use axum::body::Body;
use axum::extract::Request;
use bsnext_input::route::{Route, RouteKind};
use http::Uri;
use std::ops::ControlFlow;
use tracing::trace;

#[derive(Debug)]
pub struct RouteCandidate<'a> {
    pub index: usize,
    pub route: &'a Route,
    pub body_match: Option<BodyMatch>,
    pub mirror: Option<Mirror>,
    pub injections: Option<Injections>,
    pub delay: Option<Delay>,
    pub compress: Option<Compress>,
    pub cache_prevent: Option<CachePrevent>,
    pub cors: Option<Cors>,
    pub res_headers: Option<ResHeaders>,
}

impl<'a> RouteCandidate<'a> {
    pub fn for_route(
        index: usize,
        route: &'a Route,
        req: &Request,
        uri: &Uri,
        outer_uri: &Uri,
    ) -> Self {
        let body_match = BodyMatch::new_opt(route, req, uri, outer_uri);
        let injections = Injections::new_opt(route, req, uri, outer_uri);
        let mirror = Mirror::new_opt(route, req, uri, outer_uri);
        let delay = Delay::new_opt(route, req, uri, outer_uri);
        let compress = Compress::new_opt(route, req, uri, outer_uri);
        let cache_prevent = CachePrevent::new_opt(route, req, uri, outer_uri);
        let cors = Cors::new_opt(route, req, uri, outer_uri);
        let res_headers = ResHeaders::new_opt(route, req, uri, outer_uri);

        RouteCandidate {
            index,
            body_match,
            route,
            mirror,
            injections,
            delay,
            compress,
            cache_prevent,
            cors,
            res_headers,
        }
    }
}

impl RouteCandidate<'_> {
    pub async fn try_exec(&self, body: &mut Option<Body>) -> (Option<Body>, ControlFlow<()>) {
        if let Some(body_match) = &self.body_match {
            trace!("trying to collect body because candidate needs it");
            body_match.try_exec(body, self.route).await
        } else {
            (None, ControlFlow::Continue(()))
        }
    }
}

impl RouteCandidate<'_> {
    pub fn will_proxy(&self) -> bool {
        matches!(self.route.kind, RouteKind::Proxy(..))
    }
}
