use axum::body::Body;
use axum::extract::Request;
use axum::response::Response;
use axum::Router;
use futures_util::future::BoxFuture;
use http::{HeaderName, HeaderValue};
use insta::assert_debug_snapshot;
use std::collections::BTreeMap;
use std::task::{Context, Poll};
use tower::layer::layer_fn;
use tower::{Layer, Service, ServiceBuilder, ServiceExt};

#[tokio::test]
async fn test_manual_service_impl() -> anyhow::Result<()> {
    let app =
        Router::new().layer(
            ServiceBuilder::new().service(layer_fn(|service| MyMiddleware {
                inner: service,
                headers: [("a".to_string(), "b".to_string())].into(),
            })),
        );
    let req = Request::get("/").body(Body::empty())?;
    let s = app.oneshot(req).await?;
    assert_debug_snapshot!(s);
    Ok(())
}

#[derive(Clone)]
struct MyLayer {
    headers: BTreeMap<String, String>,
}

impl<S> Layer<S> for MyLayer {
    type Service = MyMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        MyMiddleware {
            inner,
            headers: self.headers.clone(),
        }
    }
}

#[derive(Clone)]
struct MyMiddleware<S> {
    inner: S,
    headers: BTreeMap<String, String>,
}

impl<S> Service<Request> for MyMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let future = self.inner.call(request);
        let headers = self.headers.clone();
        Box::pin(async move {
            let mut response: Response = future.await?;
            let header_map = response.headers_mut();
            for (k, v) in headers {
                let hn = HeaderName::from_bytes(k.as_bytes());
                let hv = HeaderValue::from_bytes(v.as_bytes());
                if let (Ok(k), Ok(v)) = (hn, hv) {
                    header_map.insert(k, v);
                }
            }
            Ok(response)
        })
    }
}
