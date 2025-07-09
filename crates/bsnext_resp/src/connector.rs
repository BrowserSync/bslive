use crate::injector_guard::ByteReplacer;
use crate::RespMod;
use axum::extract::Request;
use bsnext_guards::route_guard::RouteGuard;
use http::{Response, Uri};

#[derive(Debug, Default)]
pub struct Connector;

impl RouteGuard for Connector {
    fn accept_req(&self, req: &Request, _outer_uri: &Uri) -> bool {
        RespMod::accepts_html(req)
    }

    fn accept_res<T>(&self, res: &Response<T>, _outer_uri: &Uri) -> bool {
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
