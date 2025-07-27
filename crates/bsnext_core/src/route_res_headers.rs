use crate::route_effect::RouteEffect;
use axum::extract::{Request, State};
use axum::response::Response;
use bsnext_input::route::Route;
use http::{HeaderName, HeaderValue, Uri};
use std::collections::BTreeMap;

#[derive(Debug, Clone)]
pub struct ResHeaders {
    headers: BTreeMap<String, String>,
}

impl ResHeaders {
    pub fn headers(&self) -> &BTreeMap<String, String> {
        &self.headers
    }
}

impl RouteEffect for ResHeaders {
    fn new_opt(
        Route { opts, .. }: &Route,
        _req: &Request,
        _uri: &Uri,
        _outer_uri: &Uri,
    ) -> Option<Self> {
        opts.headers.as_ref().map(|headers| ResHeaders {
            headers: headers.clone(),
        })
    }
}

pub async fn set_str_headers(
    State(header_map): State<BTreeMap<String, String>>,
    mut response: Response,
) -> Response {
    let headers = response.headers_mut();
    for (k, v) in header_map {
        let hn = HeaderName::from_bytes(k.as_bytes());
        let hv = HeaderValue::from_bytes(v.as_bytes());
        match (hn, hv) {
            (Ok(k), Ok(v)) => {
                tracing::debug!("did insert header `{}`: `{:?}`", k, v);
                headers.insert(k, v);
            }
            (Ok(n), Err(_e)) => {
                tracing::error!("invalid header value: `{}` for name: `{}`", v, n)
            }
            (Err(_e), Ok(..)) => {
                tracing::error!("invalid header name `{}`", k)
            }
            (Err(_e), Err(_e2)) => {
                tracing::error!("invalid header name AND value `{}:{}`", k, v)
            }
        }
    }

    response
}
