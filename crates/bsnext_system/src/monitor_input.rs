use crate::system::BsSystem;
use actix::{Actor, Addr, AsyncContext};
use bsnext_fs::{Debounce, FsEventContext};
use bsnext_input::route::WatchSpec;
use bsnext_input::InputCtx;
use bsnext_monitor::path_monitor::PathMonitor;
use bsnext_monitor::path_monitor_meta::PathMonitorMeta;
use bsnext_monitor::watchables::any_watchable::AnyWatchable;
use bsnext_monitor::watchables::path_watchable::PathWatchable;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct InputMonitor {
    #[allow(dead_code)]
    pub addr: Addr<PathMonitor>,
    pub monitor_meta: PathMonitorMeta,
    pub input_ctx: InputCtx,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct MonitorInput {
    pub path: PathBuf,
    #[allow(dead_code)]
    pub cwd: PathBuf,
    pub input_ctx: InputCtx,
}

impl actix::Handler<MonitorInput> for BsSystem {
    type Result = ();

    #[tracing::instrument(skip_all, name = "Handler->MonitorInput->BsSystem")]
    fn handle(&mut self, msg: MonitorInput, ctx: &mut Self::Context) -> Self::Result {
        let sys = ctx.address().recipient();
        let debounce = Debounce::Trailing {
            duration: Duration::from_millis(300),
        };

        let ctx = FsEventContext::for_root();
        let paths = vec![msg.path.to_path_buf()];
        let pw = PathWatchable::Any(AnyWatchable {
            dirs: paths.clone(),
            watch_spec: WatchSpec::default(),
        });

        let input_path_monitor = PathMonitor::new(
            sys,
            debounce,
            self.cwd.clone(),
            ctx,
            pw.spec().clone(),
            paths,
        );

        let meta = PathMonitorMeta::from(&input_path_monitor);

        tracing::debug!("starting input monitor");

        let input_watcher_addr = input_path_monitor.start();

        let input_monitor = InputMonitor {
            input_ctx: msg.input_ctx.clone(),
            addr: input_watcher_addr,
            monitor_meta: meta,
        };

        self.input_monitors = Some(input_monitor);
    }
}
