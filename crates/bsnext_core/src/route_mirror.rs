use crate::route_effect::RouteEffect;
use axum::body::Body;
use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use bsnext_input::route::{Route, RouteKind};
use bytes::Bytes;
use http::header::ACCEPT;
use http::Uri;
use std::ffi::OsStr;
use std::io;
use std::path::{Path, PathBuf};
use tokio::fs::{create_dir_all, File};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt;
use tracing::error;

#[derive(Debug, Clone)]
pub struct Mirror {
    #[allow(dead_code)]
    path: PathBuf,
}

impl RouteEffect for Mirror {
    fn new_opt(route: &Route, req: &Request, _uri: &Uri, _outer_uri: &Uri) -> Option<Self> {
        let css_req = req
            .headers()
            .get(ACCEPT)
            .and_then(|h| h.to_str().ok())
            .map(|c| c.contains("text/css"))
            .unwrap_or(false);

        let js_req = Path::new(req.uri().path())
            .extension()
            .is_some_and(|ext| ext == OsStr::new("js"));

        (js_req || css_req)
            .then(|| Mirror::from_route(&route.kind))
            .flatten()
    }
}

impl Mirror {
    fn from_route(r: &RouteKind) -> Option<Self> {
        match r {
            RouteKind::Proxy(proxy) => proxy.mirror().map(|path| Self { path }),
            RouteKind::Raw(_) => None,
            RouteKind::Dir(_) => None,
        }
    }
}

#[allow(dead_code)]
async fn mirror_handler(
    State(path): State<PathBuf>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<Result<Bytes, io::Error>>();
    let as_stream = UnboundedReceiverStream::from(receiver);
    let c = req.uri().clone();
    let p = path.join(c.path().strip_prefix("/").unwrap());
    let r = next.run(req).await;
    let (parts, body) = r.into_parts();
    let s = body.into_data_stream();

    tokio::spawn(async move {
        // let s = s.throttle(Duration::from_millis(10));
        tokio::pin!(s);
        create_dir_all(&p.parent().unwrap()).await.unwrap();
        let mut file = BufWriter::new(File::create(p).await.unwrap());

        while let Some(Ok(b)) = s.next().await {
            match file.write(&b).await {
                Ok(_) => {}
                Err(e) => error!(?e, "could not write"),
            };
            // match file.write("\n".as_bytes()).await {
            //     Ok(_) => {}
            //     Err(e) => error!(?e, "could not new line"),
            // };
            match file.flush().await {
                Ok(_) => {}
                Err(e) => error!(?e, "could not flush"),
            };
            match sender.send(Ok(b)) {
                Ok(_) => {}
                Err(e) => {
                    error!(?e, "sender was dropped before reading was finished");
                    error!("will break");
                    break;
                }
            };
        }
    });

    Response::from_parts(parts, Body::from_stream(as_stream))
}
