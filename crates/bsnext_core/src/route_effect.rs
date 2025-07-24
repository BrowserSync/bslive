use axum::extract::Request;
use bsnext_input::route::Route;
use http::Uri;

pub trait RouteEffect: Sized {
    fn new_opt(_route: &Route, _req: &Request, _uri: &Uri, _outer_uri: &Uri) -> Option<Self> {
        None
    }
}
