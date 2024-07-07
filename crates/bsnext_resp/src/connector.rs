use crate::injector_guard::{ByteReplacer, InjectorGuard};
use crate::RespMod;
use axum::extract::Request;
use http::Response;

#[derive(Debug, Default)]
pub struct Connector;

impl InjectorGuard for Connector {
    fn accept_req(&self, req: &Request) -> bool {
        RespMod::accepts_html(req)
    }

    fn accept_res<T>(&self, res: &Response<T>) -> bool {
        RespMod::is_html(res)
    }
}
impl ByteReplacer for Connector {
    fn apply(&self, body: &'_ str) -> Option<String> {
        let next_body = body.replace(
            "</body>",
            format!(
                "<!-- source: snippet.html-->\
                {}\
                \
                <!-- end: snippet.html-->
                </body>",
                include_str!("js/snippet.html")
            )
            .as_str(),
        );
        Some(next_body)
    }
}
