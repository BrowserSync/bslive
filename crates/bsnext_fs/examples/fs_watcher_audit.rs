//! Minimal watcher harness for `scripts/fs-notify-audit.sh`.
//!
//! Configure logging the same way on every platform via `RUST_LOG` (see tracing-subscriber
//! [`EnvFilter`](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/filter/struct.EnvFilter.html)).
//! If unset, defaults to `bsnext_fs=trace`.

use actix::{Actor, Handler};
use bsnext_fs::actor::FsWatcher;
use bsnext_fs::watch_path_handler::RequestWatchPath;
use bsnext_fs::{FsEvent, FsEventContext};
use std::path::PathBuf;
use std::time::Duration;
use tracing_subscriber::EnvFilter;

struct Sink;

impl Actor for Sink {
    type Context = actix::Context<Self>;
}

impl Handler<FsEvent> for Sink {
    type Result = ();

    fn handle(&mut self, _msg: FsEvent, _ctx: &mut Self::Context) {}
}

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    let rust_log = std::env::var("RUST_LOG").unwrap_or_else(|_| "bsnext_fs=trace".to_string());
    let filter: EnvFilter = rust_log
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid RUST_LOG: {e}"))?;

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .without_time()
        .with_ansi(true)
        .with_writer(std::io::stderr)
        .init();

    let root = PathBuf::from(
        std::env::args()
            .nth(1)
            .expect("usage: fs_watcher_audit <WATCH_ROOT>"),
    );

    let sink = Sink.start();
    let fs = FsWatcher::new(&root, FsEventContext::default(), sink.recipient());
    let addr = fs.start();
    addr.do_send(RequestWatchPath { path: root.clone() });

    eprintln!(
        "FS_AUDIT_READY root={} host={}",
        root.display(),
        rustc_host_triple()
    );
    eprintln!("FS_AUDIT_RUST_LOG={rust_log}");

    tokio::time::sleep(Duration::from_secs(600)).await;
    Ok(())
}

fn rustc_host_triple() -> String {
    let out = std::process::Command::new("rustc")
        .args(["-vV"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .map(|o| String::from_utf8_lossy(&o.stdout).into_owned())
        .unwrap_or_else(|| "(rustc -vV failed)".into());
    out.lines()
        .find_map(|l| l.strip_prefix("host: "))
        .map(str::trim)
        .unwrap_or("unknown")
        .to_string()
}
