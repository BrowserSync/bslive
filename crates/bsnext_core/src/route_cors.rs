use crate::route_effect::RouteEffect;
use axum::extract::Request;
use bsnext_input::route::{CorsOpts, Route};
use http::Uri;
use tower_http::cors::CorsLayer;

#[derive(Debug, Clone)]
pub struct Cors {
    opts: CorsOpts,
}

impl RouteEffect for Cors {
    fn new_opt(
        Route { opts, .. }: &Route,
        _req: &Request,
        _uri: &Uri,
        _outer_uri: &Uri,
    ) -> Option<Self> {
        opts.cors
            .as_ref()
            .filter(|v| **v == CorsOpts::Cors(true))
            .map(|opts| Cors { opts: opts.clone() })
    }
}

impl Cors {
    pub fn as_layer(&self) -> CorsLayer {
        match self.opts {
            CorsOpts::Cors(true) => CorsLayer::permissive(),
            CorsOpts::Cors(false) => todo!("unreachable"),
        }
    }
}
