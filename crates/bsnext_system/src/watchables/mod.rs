use crate::system::BsSystem;
use crate::watchables::any_watchable::to_any_watchables;
use crate::watchables::path_watchable::PathWatchable;
use crate::watchables::route_watchable::to_route_watchables;
use crate::watchables::server_watchable::to_server_watchables;
use actix::Addr;
use bsnext_input::{InferWatchers, Input, WatchGlobalConfig};
use monitor_path_watchables::MonitorPathWatchables;
use tracing::debug;

pub mod any_watchable;
mod handle_fs_event_grouping;
pub mod input_monitor;
pub mod monitor_path_watchables;
pub mod path_watchable;
pub mod route_watchable;
pub mod server_watchable;

impl BsSystem {
    #[tracing::instrument(skip_all, name = "BsSystem.accept_watchables")]
    pub(crate) fn accept_watchables(&mut self, input: &Input, addr: Addr<BsSystem>) {
        let route_watchables = to_route_watchables(input);
        let server_watchables = to_server_watchables(input);
        let any_watchables = to_any_watchables(input);

        let routes = route_watchables
            .iter()
            .map(|r| PathWatchable::Route(r.to_owned()));

        let servers = server_watchables
            .iter()
            .map(|w| PathWatchable::Server(w.to_owned()));

        let any = any_watchables
            .iter()
            .map(|w| PathWatchable::Any(w.to_owned()));

        let watchables: Vec<_> = match &input.config.watchers {
            WatchGlobalConfig::Enabled { infer } => match infer {
                InferWatchers::None => {
                    debug!("processing {} any watchables", any.len());
                    any.collect()
                }
                InferWatchers::Routes => {
                    debug!("processing {} route watchables", routes.len());
                    debug!("processing {} any watchables", any.len());
                    routes.chain(any).collect()
                }
                InferWatchers::Servers => {
                    debug!("processing {} server watchables", servers.len());
                    debug!("processing {} any watchables", any.len());
                    servers.chain(any).collect()
                }
                InferWatchers::RoutesAndServers => {
                    debug!("processing {} route watchables", routes.len());
                    debug!("processing {} server watchables", servers.len());
                    debug!("processing {} any watchables", any.len());
                    routes.chain(servers).chain(any).collect()
                }
            },
            WatchGlobalConfig::Disabled => vec![],
        };

        let cwd = self.cwd.clone();
        debug!(
            "{} watchables to add, cwd: {}",
            watchables.len(),
            cwd.display()
        );
        let msg = MonitorPathWatchables { watchables, cwd };
        addr.do_send(msg);
    }
}
