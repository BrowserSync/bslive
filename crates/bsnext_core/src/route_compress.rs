use crate::route_effect::RouteEffect;
use crate::server::router::ProxyResponseEncoding;
use axum::extract::Request;
use bsnext_input::route::{CompType, CompressionOpts, Route, RouteKind};
use http::header::CONTENT_ENCODING;
use http::Uri;
use tower_http::compression::{CompressionLayer, Predicate};

#[derive(Debug, Clone)]
pub struct Compress {
    opts: Option<CompressionOpts>,
}

impl Compress {
    pub fn for_proxy(&self) -> bool {
        self.opts.is_none()
    }
}

impl RouteEffect for Compress {
    fn new_opt(
        Route { kind, opts, .. }: &Route,
        _req: &Request,
        _uri: &Uri,
        _outer_uri: &Uri,
    ) -> Option<Self> {
        let from_proxy = matches!(kind, RouteKind::Proxy(..));
        let opts = match &opts.compression {
            Some(CompressionOpts::Bool(false)) => None,
            Some(opts) => Some(opts.clone()),
            None => None,
        };
        // always try if it's a proxy
        if from_proxy {
            Some(Compress { opts: None })
        } else {
            opts.map(|opts| Compress { opts: Some(opts) })
        }
    }
}

impl Predicate for Compress {
    fn should_compress<B>(&self, response: &http::Response<B>) -> bool
    where
        B: http_body::Body,
    {
        let prev_marked = match (
            response.extensions().get::<ProxyResponseEncoding>(),
            response.headers().get(CONTENT_ENCODING),
        ) {
            // if the original CONTENT_ENCODING header was removed
            (Some(..), None) => true,
            _ => false,
        };

        prev_marked
    }
}
impl Compress {
    pub fn as_proxy_layer(&self) -> CompressionLayer<Compress> {
        CompressionLayer::new().compress_when(Compress {
            opts: self.opts.clone(),
        })
    }
    pub fn as_any_layer(&self) -> CompressionLayer {
        let opts = self.opts.as_ref().expect("unreachable");
        match &opts {
            CompressionOpts::Bool(false) => todo!("a bug?"),
            CompressionOpts::Bool(true) => CompressionLayer::new(),
            CompressionOpts::CompType(comp_type) => match comp_type {
                CompType::Gzip => CompressionLayer::new()
                    .gzip(true)
                    .no_br()
                    .no_deflate()
                    .no_zstd(),
                CompType::Br => CompressionLayer::new()
                    .br(true)
                    .no_gzip()
                    .no_deflate()
                    .no_zstd(),
                CompType::Deflate => CompressionLayer::new()
                    .deflate(true)
                    .no_gzip()
                    .no_br()
                    .no_zstd(),
                CompType::Zstd => CompressionLayer::new()
                    .zstd(true)
                    .no_gzip()
                    .no_deflate()
                    .no_br(),
            },
        }
    }
}
