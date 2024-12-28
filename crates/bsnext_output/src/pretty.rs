use crate::OutputWriter;
use bsnext_dto::internal::{ChildResult, InternalEvents};
use bsnext_dto::{
    ExternalEventsDTO, FileChangedDTO, FilesChangedDTO, InputAcceptedDTO, ServerIdentityDTO,
    ServersChangedDTO, StoppedWatchingDTO, WatchingDTO,
};
use bsnext_input::InputError;
use std::io::Write;
use std::marker::PhantomData;
use std::path::PathBuf;

pub struct PrettyPrint;

impl OutputWriter for PrettyPrint {
    fn handle_external_event<W: Write>(
        &self,
        sink: &mut W,
        evt: &ExternalEventsDTO,
    ) -> anyhow::Result<()> {
        match &evt {
            ExternalEventsDTO::ServersChanged(servers_started) => {
                print_servers_changed(sink, servers_started)
            }
            ExternalEventsDTO::Watching(watching) => print_watching(sink, watching),
            ExternalEventsDTO::WatchingStopped(watching) => print_stopped_watching(sink, watching),
            ExternalEventsDTO::InputAccepted(input_accepted) => {
                print_input_accepted(sink, input_accepted)
            }
            ExternalEventsDTO::FileChanged(file_changed) => print_file_changed(sink, file_changed),
            ExternalEventsDTO::FilesChanged(files_changed) => {
                print_files_changed(sink, files_changed)
            }
            ExternalEventsDTO::InputFileChanged(file_changed) => {
                print_input_file_changed(sink, file_changed)
            }
        }
    }

    fn handle_internal_event<W: Write>(
        &self,
        sink: &mut W,
        evt: InternalEvents,
    ) -> anyhow::Result<()> {
        match evt {
            InternalEvents::ServersChanged {
                server_resp: _,
                child_results,
            } => {
                let lines = print_server_updates(&child_results);
                for x in lines {
                    if let Err(e) = writeln!(sink, "{x}") {
                        tracing::error!(?e);
                    }
                }
            }
            InternalEvents::InputError(InputError::BsLiveRules(bs_rules)) => {
                let n = miette::GraphicalReportHandler::new();
                let mut inner = String::new();
                n.render_report(&mut inner, &bs_rules).expect("write?");
                writeln!(sink, "{}", inner)?;
            }
            InternalEvents::InputError(err) => {
                writeln!(sink, "{}", err)?;
            }
            InternalEvents::StartupError(err) => {
                writeln!(sink, "{}", err)?;
            }
        }
        Ok(())
    }
}

// const prefix: ANSIGenericString<str> = ansi_term::Color::Red.paint("[bslive]");

trait LineState {}
struct Line<T: LineState = Orig> {
    _state: PhantomData<T>,
}
struct Orig;
impl LineState for Orig {}
struct Prefixed;
impl LineState for Prefixed {}

impl Line<Orig> {
    pub fn prefixed() -> Line<Prefixed> {
        Line {
            _state: PhantomData,
        }
    }
}
impl Line<Orig> {
    // pub fn unprefixed() -> Line<Unprefixed> {
    //     Line {
    //         indent: Indent::None,
    //         _state: PhantomData,
    //     }
    // }
}
impl Line<Prefixed> {
    pub fn info(self, str: &str) -> String {
        format!("[bslive] {}", ansi_term::Color::Cyan.paint(str))
    }
}

pub fn print_file_changed<W: Write>(w: &mut W, evt: &FileChangedDTO) -> anyhow::Result<()> {
    writeln!(w, "[change] {}", evt.path)?;
    Ok(())
}

pub fn print_files_changed<W: Write>(w: &mut W, evt: &FilesChangedDTO) -> anyhow::Result<()> {
    match evt.paths.len() {
        0..=2 => {
            writeln!(w, "[change:multi] {}", short_file_list(&evt.paths))?;
        }
        3.. => {
            let other = evt.paths.len() - 2;
            let subset = evt.paths.iter().take(2).collect::<Vec<_>>();
            writeln!(
                w,
                "[change:multi] {} (and {} others)",
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

pub fn print_input_file_changed<W: Write>(w: &mut W, evt: &FileChangedDTO) -> anyhow::Result<()> {
    writeln!(w, "[change:input] {}", evt.path)?;
    Ok(())
}

pub fn print_input_accepted<W: Write>(w: &mut W, evt: &InputAcceptedDTO) -> anyhow::Result<()> {
    writeln!(w, "[input] {}", evt.path)?;
    Ok(())
}

pub fn print_watching<W: Write>(w: &mut W, evt: &WatchingDTO) -> anyhow::Result<()> {
    for x in &evt.paths {
        writeln!(w, "[watching {}] {}", evt.debounce, x)?;
    }
    Ok(())
}

pub fn print_stopped_watching<W: Write>(w: &mut W, evt: &StoppedWatchingDTO) -> anyhow::Result<()> {
    for x in &evt.paths {
        writeln!(w, "[watching:stopped] {}", x)?;
    }
    Ok(())
}

fn print_servers_changed<W>(w: &mut W, servers_started: &ServersChangedDTO) -> anyhow::Result<()>
where
    W: Write,
{
    let ServersChangedDTO {
        servers_resp,
        // changeset,
    } = servers_started;

    for server_dto in &servers_resp.servers {
        match &server_dto.identity {
            ServerIdentityDTO::Both { name, .. } => {
                writeln!(w, "[server] [{}] http://{}", name, server_dto.socket_addr)?;
            }
            ServerIdentityDTO::Address { .. } => {
                writeln!(w, "[server] http://{}", server_dto.socket_addr)?;
            }
            ServerIdentityDTO::Named { name } => {
                writeln!(w, "[server] [{}] http://{}", name, &server_dto.socket_addr)?
            }
        }
    }
    Ok(())
}

pub fn print_server_updates(evts: &[ChildResult]) -> Vec<String> {
    evts.iter()
        .flat_map(|r| match r {
            ChildResult::Created(created) => {
                vec![format!(
                    "[created] {}",
                    server_display(
                        &ServerIdentityDTO::from(&created.server_handler.identity),
                        &created.server_handler.socket_addr.to_string()
                    ),
                )]
            }
            ChildResult::Stopped(stopped) => {
                vec![format!("[stopped] {}", iden(stopped))]
            }
            ChildResult::CreateErr(errored) => {
                vec![format!(
                    "[server] errored... {:?} {} ",
                    iden(&errored.identity),
                    errored.server_error
                )]
            }
            ChildResult::Patched(child) => {
                let mut lines = vec![];
                // todo: determine WHICH changes were actually applied (instead of saying everything was patched)
                for x in &child.route_change_set.changed {
                    lines.push(format!(
                        "[patched] {} {:?}",
                        iden(&child.server_handler.identity),
                        x
                    ));
                }
                for x in &child.route_change_set.added {
                    lines.push(format!(
                        "[patched] {} {:?}",
                        iden(&child.server_handler.identity),
                        x
                    ));
                }
                lines
            }
            ChildResult::PatchErr(errored) => {
                vec![format!(
                    "[patch] error {} {} ",
                    iden(&errored.identity),
                    errored.patch_error
                )]
            }
        })
        .collect()
}

pub fn iden(identity_dto: impl Into<ServerIdentityDTO>) -> String {
    match identity_dto.into() {
        ServerIdentityDTO::Both { name, bind_address } => format!("[{name}] {bind_address}"),
        ServerIdentityDTO::Address { bind_address } => bind_address.to_string(),
        ServerIdentityDTO::Named { name } => format!("[{name}]"),
    }
}

pub fn server_display(identity_dto: &ServerIdentityDTO, socket_addr: &str) -> String {
    match &identity_dto {
        ServerIdentityDTO::Both { name, .. } => {
            format!("[server] [{}] http://{}", name, socket_addr)
        }
        ServerIdentityDTO::Address { .. } => {
            format!("[server] http://{}", socket_addr)
        }
        ServerIdentityDTO::Named { name } => {
            format!("[server] [{}] http://{}", name, &socket_addr)
        }
    }
}
