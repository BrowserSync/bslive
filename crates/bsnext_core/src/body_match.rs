use crate::match_json::{match_json_body, NeedsJsonGuard};
use crate::route_effect::RouteEffect;
use axum::body::Body;
use axum::extract::Request;
use bsnext_guards::route_guard::RouteGuard;
use bsnext_input::route::Route;
use http::Uri;
use std::ops::ControlFlow;

#[derive(Debug, Clone)]
pub enum BodyMatch {
    Json,
}

impl RouteEffect for BodyMatch {
    fn new_opt(route: &Route, req: &Request, _uri: &Uri, outer_uri: &Uri) -> Option<Self> {
        route.when_body.as_ref().and_then(|body| {
            if NeedsJsonGuard(body).accept_req(req, outer_uri) {
                return Some(Self::Json);
            }
            None
        })
    }
}

impl BodyMatch {
    pub async fn try_exec(
        &self,
        body: &mut Option<Body>,
        route: &Route,
    ) -> (Option<Body>, ControlFlow<()>) {
        match &self {
            BodyMatch::Json => match_json_body(body, route).await,
        }
    }
}

// impl RouteGuard for BodyMatch {
//     fn accept_req(&self, req: &Request, outer_uri: &Uri) -> bool {
//
//     }
//
//     fn accept_res<T>(&self, res: &Response<T>, outer_uri: &Uri) -> bool {
//         todo!()
//     }
// }
