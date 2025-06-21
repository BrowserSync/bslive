use crate::any_watchable::AnyWatchable;
use crate::path_monitor::{PathMonitor, PathMonitorMeta};
use crate::path_watchable::PathWatchable;
use crate::BsSystem;
use actix::{Actor, Addr, AsyncContext};
use bsnext_fs::{Debounce, FsEventContext};
use bsnext_input::route::Spec;
use bsnext_input::InputCtx;
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

        let cwd = self
            .cwd
            .as_ref()
            .map(ToOwned::to_owned)
            .expect("if this fails, lots will");

        let ctx = FsEventContext::for_root();
        let pw = PathWatchable::Any(AnyWatchable {
            dirs: vec![msg.path.to_path_buf()],
            spec: Spec::default(),
            task_list: None,
        });

        let input_path_monitor = PathMonitor::new(sys, debounce, cwd, ctx, pw);
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
