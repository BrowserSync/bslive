use axum::extract::Request;
use http::request::Parts;
use http::Response;

pub trait RouteGuard {
    fn accept_req(&self, _req: &Request) -> bool {
        true
    }
    fn accept_req_parts(&self, _parts: &Parts) -> bool {
        true
    }
    fn accept_res<T>(&self, _res: &Response<T>) -> bool {
        true
    }
}
