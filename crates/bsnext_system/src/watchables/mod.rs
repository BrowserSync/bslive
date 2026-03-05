use crate::monitor_path_watchables::MonitorPathWatchables;
use crate::system::BsSystem;
use crate::watchables::any_watchable::to_any_watchables;
use crate::watchables::path_watchable::PathWatchable;
use crate::watchables::route_watchable::to_route_watchables;
use crate::watchables::server_watchable::to_server_watchables;
use actix::Addr;
use bsnext_input::Input;
use tracing::debug;

pub mod any_watchable;
pub mod path_watchable;
pub mod route_watchable;
pub mod server_watchable;

impl BsSystem {
    #[tracing::instrument(skip_all, name = "BsSystem.accept_watchables")]
    pub(crate) fn accept_watchables(&mut self, input: &Input, addr: Addr<BsSystem>) {
        let route_watchables = to_route_watchables(input);
        let server_watchables = to_server_watchables(input);
        let any_watchables = to_any_watchables(input);

        debug!("processing {} route watchables", route_watchables.len(),);
        debug!("processing {} server watchables", server_watchables.len());
        debug!("processing {} any watchables", any_watchables.len());

        // todo: clean up this merging
        let all_watchables = route_watchables
            .iter()
            .map(|r| PathWatchable::Route(r.to_owned()));

        let servers = server_watchables
            .iter()
            .map(|w| PathWatchable::Server(w.to_owned()));

        let any = any_watchables
            .iter()
            .map(|w| PathWatchable::Any(w.to_owned()));

        let watchables = all_watchables.chain(servers).chain(any).collect::<Vec<_>>();

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
