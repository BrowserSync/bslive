use crate::route_effect::RouteEffect;
use axum::extract::{Query, Request, State};
use axum::response::Response;
use bsnext_input::route::Route;
use bsnext_query::dynamic_query_params::DynamicQueryParams;
use bsnext_resp::cache_opts::CacheOpts;
use http::header::{CACHE_CONTROL, EXPIRES, PRAGMA};
use http::{HeaderName, HeaderValue, Uri};

#[derive(Debug, Clone)]
pub struct CachePrevent {
    opts: CacheOpts,
}

impl CachePrevent {
    pub fn opts(&self) -> &CacheOpts {
        &self.opts
    }
}

impl RouteEffect for CachePrevent {
    fn new_opt(
        Route { opts, .. }: &Route,
        req: &Request,
        _uri: &Uri,
        _outer_uri: &Uri,
    ) -> Option<Self> {
        let url_params: Result<Query<DynamicQueryParams>, _> = Query::try_from_uri(req.uri());

        url_params
            .ok()
            .and_then(|Query(params)| params.cache)
            .map(|opts| CachePrevent { opts })
            .or_else(|| {
                Some(CachePrevent {
                    opts: opts.cache.clone(),
                })
            })
    }
}

impl CachePrevent {
    pub fn headers(&self) -> Vec<(HeaderName, HeaderValue)> {
        self.opts.as_headers()
    }
}

pub async fn cache_control_layer(State(opts): State<CacheOpts>, mut res: Response) -> Response {
    let headers = vec![
        (
            CACHE_CONTROL,
            HeaderValue::from_static("no-store, no-cache, must-revalidate"),
        ),
        (PRAGMA, HeaderValue::from_static("no-cache")),
        (EXPIRES, HeaderValue::from_static("0")),
    ];
    match opts {
        CacheOpts::Prevent => {
            for (name, value) in headers {
                res.headers_mut().insert(name, value);
            }
        }
        CacheOpts::Default => {
            for (name, value) in headers {
                res.headers_mut().remove(name);
            }
        }
    }
    res
}
