use axum::body::Body;
use axum::extract::Request;
use http::request::Parts;
use http::{Response, Uri};

pub trait RouteGuard {
    fn accept_req(&self, req: &Request, outer_uri: &Uri) -> bool;
    fn accept_res<T>(&self, res: &Response<T>, outer_uri: &Uri) -> bool;
    fn accept_req_parts(&self, _parts: &Parts, _outer_uri: &Uri) -> bool {
        true
    }
}

pub trait ConsumedRouteGuard {
    fn accept_req(&self, parts: &Parts, body: Body, outer_uri: &Uri) -> bool;
}
