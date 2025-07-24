use crate::match_json::NeedsJsonGuard;
use axum::extract::Request;
use axum::response::Response;
use bsnext_guards::route_guard::RouteGuard;
use bsnext_input::route::{ListOrSingle, Route};
use bsnext_input::when_guard::{HasGuard, WhenGuard};
use http::request::Parts;
use http::uri::PathAndQuery;
use http::Uri;
use tracing::trace;

pub struct RouteMatch<'a>(pub &'a Route);

impl RouteMatch<'_> {
    pub fn route_match(
        &self,
        req: &Request,
        outer_uri: &Uri,
        path: &str,
        pq: Option<&PathAndQuery>,
        parts: &Parts,
    ) -> bool {
        let route = self.0;
        trace!(?route.kind);

        // early checks from parts only
        let can_serve: bool = route
            .when
            .as_ref()
            .map(|when| match &when {
                ListOrSingle::WhenOne(when) => match_one(when, outer_uri, path, pq, parts),
                ListOrSingle::WhenMany(many) => many
                    .iter()
                    .all(|when| match_one(when, outer_uri, path, pq, parts)),
            })
            .unwrap_or(true);

        // if this routes wants to inspect the body, check it was a POST
        let can_consume = match &route.when_body {
            None => true,
            Some(when_body) => {
                let consuming = NeedsJsonGuard(when_body).accept_req(req, outer_uri);
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
