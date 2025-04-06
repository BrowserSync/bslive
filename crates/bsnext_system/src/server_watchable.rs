use crate::runner::Runner;
use bsnext_input::route::{RunOpt, Spec};
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::Input;
use std::path::PathBuf;

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct ServerWatchable {
    pub server_identity: ServerIdentity,
    pub dir: PathBuf,
    pub spec: Spec,
    pub runner: Option<Runner>,
}

pub fn to_server_watchables(input: &Input) -> Vec<ServerWatchable> {
    input
        .servers
        .iter()
        .flat_map(|server_config| {
            server_config.watchers.iter().map(|watcher| {
                let spec = watcher.opts.as_ref();
                let runner = to_runner(spec);
                ServerWatchable {
                    server_identity: server_config.identity.clone(),
                    dir: PathBuf::from(&watcher.dir),
                    spec: watcher.opts.clone().unwrap_or_default(),
                    runner,
                }
            })
        })
        .collect()
}

pub fn to_runner(spec: Option<&Spec>) -> Option<Runner> {
    let run = spec.as_ref()?.run.as_ref()?;
    match &run {
        RunOpt::All { all } if !all.is_empty() => Some(Runner::all_from(all)),
        RunOpt::Seq(seq) if !seq.is_empty() => Some(Runner::seq_from(seq)),
        _ => None,
    }
}
