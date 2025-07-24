use crate::body_match::BodyMatch;
use crate::route_compress::Compress;
use crate::route_delay::Delay;
use crate::route_effect::RouteEffect;
use crate::route_injections::Injections;
use crate::route_mirror::Mirror;
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

        RouteCandidate {
            index,
            body_match,
            route,
            mirror,
            injections,
            delay,
            compress,
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
