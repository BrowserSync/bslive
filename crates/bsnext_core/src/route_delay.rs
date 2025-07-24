use crate::route_effect::RouteEffect;
use axum::extract::{Query, Request, State};
use axum::middleware::Next;
use axum::response::IntoResponse;
use bsnext_input::route::{DelayKind, DelayOpts, Route};
use bsnext_query::dynamic_query_params::DynamicQueryParams;
use http::Uri;
use std::convert::Infallible;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone)]
pub struct Delay {
    opts: DelayOpts,
}

impl Delay {
    pub fn opts(&self) -> &DelayOpts {
        &self.opts
    }
}

impl RouteEffect for Delay {
    fn new_opt(
        Route { opts, .. }: &Route,
        req: &Request,
        _uri: &Uri,
        _outer_uri: &Uri,
    ) -> Option<Self> {
        let from_route = opts.delay.as_ref().map(ToOwned::to_owned);
        let url_params: Result<Query<DynamicQueryParams>, _> = Query::try_from_uri(req.uri());

        //
        let from_url = url_params
            .ok()
            .and_then(|Query(params)| params.delay)
            .map(|ms| DelayOpts::Delay(DelayKind::Ms(ms)));

        // URL takes precedence over route
        from_url.or(from_route).map(|opts| Self { opts })
    }
}

pub async fn delay_mw(
    State(delay_opts): State<DelayOpts>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    match delay_opts {
        DelayOpts::Delay(DelayKind::Ms(ms)) => {
            let res = next.run(req).await;
            sleep(Duration::from_millis(ms)).await;
            Ok::<_, Infallible>(res)
        }
    }
}
