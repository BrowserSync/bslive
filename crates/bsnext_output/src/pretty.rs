use crate::OutputWriter;
use bsnext_dto::{
    ExternalEvents, FileChanged, FilesChangedDTO, IdentityDTO, InputAccepted, ServerChange,
    ServerChangeSetItem, ServersStarted, StoppedWatching, Watching,
};
use std::io::Write;
use std::path::PathBuf;

pub struct PrettyPrint;

impl OutputWriter for PrettyPrint {
    fn handle_event<W: Write>(&self, sink: &mut W, evt: &ExternalEvents) -> anyhow::Result<()> {
        match &evt {
            ExternalEvents::ServersStarted(servers_started) => {
                print_server_started(sink, servers_started)
            }
            ExternalEvents::StartupFailed(_input_err) => {
                unreachable!("StartupFailed")
            }
            ExternalEvents::Watching(watching) => print_watching(sink, watching),
            ExternalEvents::WatchingStopped(watching) => print_stopped_watching(sink, watching),
            ExternalEvents::InputAccepted(input_accepted) => {
                print_input_accepted(sink, input_accepted)
            }
            ExternalEvents::FileChanged(file_changed) => print_file_changed(sink, file_changed),
            ExternalEvents::FilesChanged(files_changed) => print_files_changed(sink, files_changed),
            ExternalEvents::InputFileChanged(file_changed) => {
                print_input_file_changed(sink, file_changed)
            }
        }
    }
}

pub fn print_file_changed<W: Write>(w: &mut W, evt: &FileChanged) -> anyhow::Result<()> {
    writeln!(w, "[change] {}", evt.path)?;
    Ok(())
}

pub fn print_files_changed<W: Write>(w: &mut W, evt: &FilesChangedDTO) -> anyhow::Result<()> {
    match evt.paths.len() {
        0..=2 => {
            writeln!(w, "[multi-change] {}", short_file_list(&evt.paths))?;
        }
        3.. => {
            let other = evt.paths.len() - 2;
            let subset = evt.paths.iter().take(2).collect::<Vec<_>>();
            writeln!(
                w,
                "[multi-change] {} (and {} others)",
                short_file_list(&subset),
                other
            )?;
        }
    }
    Ok(())
}

fn short_file_list<A: AsRef<str>>(paths: &[A]) -> String {
    let file_names = paths
        .iter()
        .filter_map(|p| {
            PathBuf::from(p.as_ref())
                .file_name()
                .map(|filename| filename.to_string_lossy().to_string())
        })
        .collect::<Vec<_>>();
    file_names.join(", ")
}

pub fn print_input_file_changed<W: Write>(w: &mut W, evt: &FileChanged) -> anyhow::Result<()> {
    writeln!(w, "[change:input] {}", evt.path)?;
    Ok(())
}

pub fn print_input_accepted<W: Write>(w: &mut W, evt: &InputAccepted) -> anyhow::Result<()> {
    writeln!(w, "[input] {}", evt.path)?;
    Ok(())
}

pub fn print_watching<W: Write>(w: &mut W, evt: &Watching) -> anyhow::Result<()> {
    for x in &evt.paths {
        writeln!(w, "[watching {}] {}", evt.debounce, x)?;
    }
    Ok(())
}

pub fn print_stopped_watching<W: Write>(w: &mut W, evt: &StoppedWatching) -> anyhow::Result<()> {
    for x in &evt.paths {
        writeln!(w, "[watching:stopped] {}", x)?;
    }
    Ok(())
}

fn print_server_started<W>(w: &mut W, servers_started: &ServersStarted) -> anyhow::Result<()>
where
    W: Write,
{
    let ServersStarted {
        servers_resp,
        changeset,
    } = servers_started;

    for ServerChangeSetItem { change, identity } in &changeset.items {
        let running = servers_resp
            .servers
            .iter()
            .find(|x| x.identity == *identity);
        match change {
            ServerChange::Stopped { bind_address } => match &identity {
                IdentityDTO::Both { name, bind_address } => {
                    writeln!(w, "[server removed] [{name}] http://{bind_address}")?;
                }
                IdentityDTO::Address { bind_address } => {
                    writeln!(w, "[server removed] http://{bind_address}")?;
                }
                IdentityDTO::Named { name } => {
                    writeln!(w, "[server removed] [{name}] http://{}", bind_address)?;
                }
            },
            ServerChange::Started => match &identity {
                IdentityDTO::Both { name, bind_address } => {
                    if running.is_some() {
                        writeln!(w, "[server added] [{}] http://{}", name, bind_address)?;
                    }
                }
                IdentityDTO::Address { bind_address } => {
                    if running.is_some() {
                        writeln!(w, "[server added] http://{}", bind_address)?;
                    }
                }
                IdentityDTO::Named { name } => {
                    if let Some(running) = running {
                        writeln!(
                            w,
                            "[server added] [{}] http://{}",
                            name, &running.socket_addr
                        )?;
                    } else {
                        unreachable!("?");
                    }
                }
            },
            ServerChange::Patched => {}
        }
    }
    Ok(())
}
