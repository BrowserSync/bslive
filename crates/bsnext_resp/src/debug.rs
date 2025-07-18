use crate::injector_guard::ByteReplacer;
use axum::extract::Request;
use bsnext_guards::route_guard::RouteGuard;
use http::{Response, Uri};

#[derive(Debug, Default)]
pub struct Debug;

impl RouteGuard for Debug {
    fn accept_req(&self, req: &Request, _uri: &Uri) -> bool {
        req.uri().path().contains("core.css")
    }

    fn accept_res<T>(&self, _res: &Response<T>, _uri: &Uri) -> bool {
        true
    }
}
impl ByteReplacer for Debug {
    fn apply(&self, body: &'_ str) -> Option<String> {
        let next = format!("{}{}", body, "/** hey! */");
        Some(next)
    }
}
