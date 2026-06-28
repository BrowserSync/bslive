use crate::FsGroup;
use crate::watchables::any_watchable::to_any_watchables;
use crate::watchables::path_watchable::PathWatchable;
use crate::watchables::route_watchable::to_route_watchables;
use crate::watchables::server_watchable::to_server_watchables;
use actix::Recipient;
use bsnext_input::{InferWatchers, Input, WatchGlobalConfig};
use std::path::PathBuf;
use tracing::debug;

pub mod any_watchable;
pub mod path_watchable;
pub mod route_watchable;
pub mod server_watchable;

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct MonitorPathWatchables {
    pub watchables: Vec<PathWatchable>,
    pub cwd: PathBuf,
    pub recipient: Recipient<FsGroup>,
}

#[tracing::instrument(skip_all, name = "accept_watchables")]
pub fn accept_watchables(
    cwd: PathBuf,
    input: &Input,
    recipient: Recipient<FsGroup>,
) -> MonitorPathWatchables {
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

    debug!(
        "{} watchables to add, cwd: {}",
        watchables.len(),
        cwd.display()
    );

    MonitorPathWatchables {
        watchables,
        cwd,
        recipient,
    }
}
