use crate::OutputWriter;
use bsnext_dto::internal::{ChildResult, InternalEvents};
use bsnext_dto::{
    ExternalEvents, FileChanged, FilesChangedDTO, IdentityDTO, InputAccepted, InputErrorDTO,
    ServersChanged, StartupErrorDTO, StartupEvent, StoppedWatching, Watching,
};
use std::io::Write;
use std::marker::PhantomData;
use std::path::PathBuf;

pub struct PrettyPrint;

impl OutputWriter for PrettyPrint {
    fn handle_external_event<W: Write>(
        &self,
        sink: &mut W,
        evt: &ExternalEvents,
    ) -> anyhow::Result<()> {
        match &evt {
            ExternalEvents::ServersChanged(servers_started) => {
                print_servers_changed(sink, servers_started)
            }
            ExternalEvents::InputError(input_err) => {
                print_input_error(sink, Indent::None, input_err)
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

    fn handle_internal_event<W: Write>(
        &self,
        sink: &mut W,
        evt: InternalEvents,
    ) -> anyhow::Result<()> {
        match evt {
            InternalEvents::ServersChanged {
                server_resp,
                child_results: _,
            } => print_servers_changed(
                sink,
                &ServersChanged {
                    servers_resp: server_resp,
                },
            ),
        }
    }

    fn handle_startup_event<W: Write>(
        &self,
        sink: &mut W,
        evt: &StartupEvent,
    ) -> anyhow::Result<()> {
        match evt {
            StartupEvent::Started => {
                writeln!(sink, "{}", Line::prefixed().info("started..."))?;
            }
            StartupEvent::FailedStartup(err) => {
                writeln!(
                    sink,
                    "{}",
                    Line::prefixed().info("An error prevented startup!")
                )?;
                writeln!(sink)?;
                match err {
                    StartupErrorDTO::InputError(input_err) => {
                        print_input_error(sink, Indent::Some(4), input_err)?;
                    }
                }
            }
        }
        Ok(())
    }
}

// const prefix: ANSIGenericString<str> = ansi_term::Color::Red.paint("[bslive]");

trait LineState {}
struct Line<T: LineState = Orig> {
    indent: Indent,
    _state: PhantomData<T>,
}
struct Orig;
impl LineState for Orig {}
struct Prefixed;
impl LineState for Prefixed {}

struct Unprefixed;
impl LineState for Unprefixed {}
impl Line<Orig> {
    pub fn prefixed() -> Line<Prefixed> {
        Line {
            indent: Indent::None,
            _state: PhantomData,
        }
    }
}
impl Line<Orig> {
    pub fn unprefixed() -> Line<Unprefixed> {
        Line {
            indent: Indent::None,
            _state: PhantomData,
        }
    }
}
impl Line<Prefixed> {
    pub fn info(self, str: &str) -> String {
        format!("[bslive] {}", ansi_term::Color::Cyan.paint(str))
    }
}
impl Line<Unprefixed> {
    pub fn indent(self, size: Indent) -> Self {
        Self {
            indent: size,
            _state: PhantomData,
        }
    }
    pub fn error(self, str: &str) -> String {
        let coloured = ansi_term::Color::Red.paint(str);
        indent::indent_all_by(self.indent.indent_size(), coloured.to_string())
    }
}

pub fn print_file_changed<W: Write>(w: &mut W, evt: &FileChanged) -> anyhow::Result<()> {
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

enum Indent {
    None,
    Some(usize),
}

impl Indent {
    pub fn indent_size(&self) -> usize {
        match self {
            Indent::None => 0,
            Indent::Some(size) => *size,
        }
    }
}

fn print_input_error<W: Write>(
    w: &mut W,
    indent: Indent,
    evt: &InputErrorDTO,
) -> anyhow::Result<()> {
    let v = match evt {
        InputErrorDTO::MissingInputs(evt) => evt,
        InputErrorDTO::InvalidInput(evt) => evt,
        InputErrorDTO::NotFound(evt) => evt,
        InputErrorDTO::InputWriteError(evt) => evt,
        InputErrorDTO::PathError(evt) => evt,
        InputErrorDTO::PortError(evt) => evt,
        InputErrorDTO::DirError(evt) => evt,
        InputErrorDTO::YamlError(evt) => evt,
        InputErrorDTO::MarkdownError(evt) => evt,
        InputErrorDTO::Io(evt) => evt,
        InputErrorDTO::UnsupportedExtension(evt) => evt,
        InputErrorDTO::MissingExtension(evt) => evt,
        InputErrorDTO::EmptyInput(evt) => evt,
    };
    writeln!(w, "{}", Line::unprefixed().indent(indent).error(v))?;
    Ok(())
}

pub fn print_stopped_watching<W: Write>(w: &mut W, evt: &StoppedWatching) -> anyhow::Result<()> {
    for x in &evt.paths {
        writeln!(w, "[watching:stopped] {}", x)?;
    }
    Ok(())
}

fn print_servers_changed<W>(w: &mut W, servers_started: &ServersChanged) -> anyhow::Result<()>
where
    W: Write,
{
    let ServersChanged {
        servers_resp,
        // changeset,
    } = servers_started;

    for server_dto in &servers_resp.servers {
        match &server_dto.identity {
            IdentityDTO::Both { name, .. } => {
                writeln!(w, "[server] [{}] http://{}", name, server_dto.socket_addr)?;
            }
            IdentityDTO::Address { .. } => {
                writeln!(w, "[server] http://{}", server_dto.socket_addr)?;
            }
            IdentityDTO::Named { name } => {
                writeln!(w, "[server] [{}] http://{}", name, &server_dto.socket_addr)?
            }
        }
    }

    // for ServerChangeSetItem { change, identity } in &changeset.items {
    //     let running = servers_resp
    //         .servers
    //         .iter()
    //         .find(|x| x.identity == *identity);
    //     match change {
    //         ServerChange::Stopped { bind_address } => match &identity {
    //             IdentityDTO::Both { name, bind_address } => {
    //                 writeln!(w, "[server removed] [{name}] http://{bind_address}")?;
    //             }
    //             IdentityDTO::Address { bind_address } => {
    //                 writeln!(w, "[server removed] http://{bind_address}")?;
    //             }
    //             IdentityDTO::Named { name } => {
    //                 writeln!(w, "[server removed] [{name}] http://{}", bind_address)?;
    //             }
    //         },
    //         ServerChange::Started => match &identity {
    //             IdentityDTO::Both { name, bind_address } => {
    //                 if running.is_some() {
    //                     writeln!(w, "[server added] [{}] http://{}", name, bind_address)?;
    //                 }
    //             }
    //             IdentityDTO::Address { bind_address } => {
    //                 if running.is_some() {
    //                     writeln!(w, "[server added] http://{}", bind_address)?;
    //                 }
    //             }
    //             IdentityDTO::Named { name } => {
    //                 if let Some(running) = running {
    //                     writeln!(
    //                         w,
    //                         "[server added] [{}] http://{}",
    //                         name, &running.socket_addr
    //                     )?;
    //                 } else {
    //                     unreachable!("?");
    //                 }
    //             }
    //         },
    //         ServerChange::Patched => {}
    //         ServerChange::Errored { error } => {
    //             writeln!(w, "[❌ server failed] {} {}", iden(identity), error)?;
    //         }
    //     }
    // }
    Ok(())
}

pub fn print_server_updates(evts: &[ChildResult]) -> Vec<String> {
    evts.iter()
        .map(|r| match r {
            ChildResult::Created(created) => {
                vec![format!(
                    "[created] {}",
                    server_display(
                        &IdentityDTO::from(&created.server_handler.identity),
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
        .flatten()
        .collect()
}

pub fn iden(identity_dto: impl Into<IdentityDTO>) -> String {
    match identity_dto.into() {
        IdentityDTO::Both { name, bind_address } => format!("[{name}] {bind_address}"),
        IdentityDTO::Address { bind_address } => bind_address.to_string(),
        IdentityDTO::Named { name } => format!("[{name}]"),
    }
}

pub fn server_display(identity_dto: &IdentityDTO, socket_addr: &str) -> String {
    match &identity_dto {
        IdentityDTO::Both { name, .. } => {
            format!("[server] [{}] http://{}", name, socket_addr)
        }
        IdentityDTO::Address { .. } => {
            format!("[server] http://{}", socket_addr)
        }
        IdentityDTO::Named { name } => {
            format!("[server] [{}] http://{}", name, &socket_addr)
        }
    }
}
