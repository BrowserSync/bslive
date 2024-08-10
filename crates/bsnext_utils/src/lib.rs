use axum::body::Body;
use http::response::Parts;
use http::{HeaderMap, Request, Uri};
use http_body_util::BodyExt;
use hyper_tls::HttpsConnector;
use hyper_util::client::legacy::connect::HttpConnector;
use hyper_util::client::legacy::Client;
use hyper_util::rt::TokioExecutor;
use std::net::SocketAddr;

pub async fn req_to_str(
    socket_addr: SocketAddr,
    uri: &str,
    headers: fn(&mut HeaderMap) -> &mut HeaderMap,
) -> anyhow::Result<(Parts, String)> {
    let https = HttpsConnector::new();
    let client: Client<HttpsConnector<HttpConnector>, Body> =
        Client::builder(TokioExecutor::new()).build(https);

    let uri = Uri::builder()
        .scheme("http")
        .authority(socket_addr.to_string())
        .path_and_query(uri)
        .build()
        .expect("valid uri");

    let mut r = Request::builder().uri(uri).body(Body::empty()).unwrap();
    headers(r.headers_mut());

    let resp = client.request(r).await.expect("result");

    let (parts, body) = resp.into_parts();

    let bytes = match body.collect().await {
        Ok(c) => c.to_bytes(),
        Err(_) => unreachable!("cannot error"),
    };

    match std::str::from_utf8(&bytes[..]) {
        Ok(s) => Ok((parts, String::from(s))),
        Err(_e) => Err(anyhow::anyhow!("oops")),
    }
}
