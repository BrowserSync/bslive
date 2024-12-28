use crate::injector_guard::ByteReplacer;
use crate::RespMod;
use axum::extract::Request;
use bsnext_guards::route_guard::RouteGuard;
use http::Response;

#[derive(Debug, Default)]
pub struct JsConnector;

impl RouteGuard for JsConnector {
    fn accept_req(&self, _req: &Request) -> bool {
        true
    }

    fn accept_res<T>(&self, res: &Response<T>) -> bool {
        let is_js = RespMod::is_js(res);
        tracing::trace!("is_js: {}", is_js);
        is_js
    }
}
impl ByteReplacer for JsConnector {
    fn apply(&self, body: &'_ str) -> Option<String> {
        let footer = format!(
            r#"{body};
            // this was injected by the Browsersync Live Js Connector
            ;import('/__bs_js').catch(console.error);
        "#
        );
        Some(footer)
    }
}
