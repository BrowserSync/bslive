use axum::extract::Request;
use http::Response;

pub trait RouteGuard {
    fn accept_req(&self, req: &Request) -> bool;
    fn accept_res<T>(&self, res: &Response<T>) -> bool;
}
