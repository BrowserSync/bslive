use crate::BsSystem;
use actix::{Actor, Addr, AsyncContext};
use bsnext_fs::actor::FsWatcher;
use bsnext_fs::watch_path_handler::RequestWatchPath;
use bsnext_fs::Debounce;
use bsnext_input::InputCtx;
use std::path::PathBuf;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct InputMonitor {
    pub addr: Addr<FsWatcher>,
    pub ctx: InputCtx,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct MonitorInput {
    pub path: PathBuf,
    pub cwd: PathBuf,
    pub ctx: InputCtx,
}

impl actix::Handler<MonitorInput> for BsSystem {
    type Result = ();

    fn handle(&mut self, msg: MonitorInput, ctx: &mut Self::Context) -> Self::Result {
        let mut input_watcher = bsnext_fs::actor::FsWatcher::for_input(&msg.cwd, 0);

        // todo: does this need to be configurable (eg: by main config)?
        input_watcher.with_debounce(Debounce::Trailing {
            duration: Duration::from_millis(300),
        });

        tracing::debug!("starting input monitor");

        let input_watcher_addr = input_watcher.start();
        let input_monitor = InputMonitor {
            addr: input_watcher_addr.clone(),
            ctx: msg.ctx.clone(),
        };
        self.input_monitors = Some(input_monitor);

        input_watcher_addr.do_send(RequestWatchPath {
            recipients: vec![ctx.address().recipient()],
            path: msg.path.to_path_buf(),
        });
    }
}
