use crate::injector_guard::{ByteReplacer, InjectorGuard};
use axum::extract::Request;
use http::Response;

#[derive(Debug, Default)]
pub struct Debug;

impl InjectorGuard for Debug {
    fn accept_req(&self, req: &Request) -> bool {
        req.uri().path().contains("core.css")
    }

    fn accept_res<T>(&self, _res: &Response<T>) -> bool {
        true
    }
}
impl ByteReplacer for Debug {
    fn apply(&self, body: &'_ str) -> Option<String> {
        let next = format!("{}{}", body, "/** hey! */");
        Some(next)
    }
}
