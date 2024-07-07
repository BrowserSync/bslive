use axum::extract::Request;
use bytes::Bytes;
use http::{HeaderMap, Response};

pub trait InjectorGuard {
    fn accept_req(&self, req: &Request) -> bool;
    fn accept_res<T>(&self, res: &Response<T>) -> bool;
}

pub trait ByteReplacer: InjectorGuard {
    fn apply(&self, body: &'_ str) -> Option<String>;

    fn replace_bytes(
        &self,
        incoming: &Bytes,
        _req_headers: &HeaderMap,
        _res_headers: &HeaderMap,
    ) -> Option<Bytes> {
        if let Ok(body) = std::str::from_utf8(incoming) {
            let next = self.apply(body);
            next.map(Bytes::from)
        } else {
            tracing::error!("incoming bytes were not UTF-8");
            None
        }
    }
}
