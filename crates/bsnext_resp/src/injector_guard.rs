use bsnext_guards::route_guard::RouteGuard;
use bytes::Bytes;
use http::HeaderMap;

pub trait ByteReplacer: RouteGuard {
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
