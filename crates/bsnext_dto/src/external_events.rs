use crate::{
    FileChangedDTO, FilesChangedDTO, InputAcceptedDTO, OutputLineDTO, ServerIdentityDTO,
    ServersChangedDTO, StderrLineDTO, StdoutLineDTO, StoppedWatchingDTO, WatchingDTO,
};
use bsnext_output::OutputWriterTrait;
use std::io::Write;
use std::path::PathBuf;
use typeshare::typeshare;

/// @discriminator kind
#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum ExternalEventsDTO {
    ServersChanged(ServersChangedDTO),
    Watching(WatchingDTO),
    WatchingStopped(StoppedWatchingDTO),
    FileChanged(FileChangedDTO),
    FilesChanged(FilesChangedDTO),
    InputFileChanged(FileChangedDTO),
    InputAccepted(InputAcceptedDTO),
    OutputLine(OutputLineDTO),
}

impl ExternalEventsDTO {
    pub fn stdout_line(line: String, prefix: Option<String>) -> Self {
        Self::OutputLine(crate::OutputLineDTO::Stdout(StdoutLineDTO { line, prefix }))
    }
    pub fn stderr_line(line: String, prefix: Option<String>) -> Self {
        Self::OutputLine(crate::OutputLineDTO::Stderr(StderrLineDTO { line, prefix }))
    }
}

impl OutputWriterTrait for ExternalEventsDTO {
    fn write_json<W: Write>(&self, sink: &mut W) -> anyhow::Result<()> {
        writeln!(sink, "{}", serde_json::to_string(&self)?)
            .map_err(|e| anyhow::anyhow!(e.to_string()))
    }

    fn write_pretty<W: Write>(&self, sink: &mut W) -> anyhow::Result<()> {
        match self {
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
            ExternalEventsDTO::OutputLine(OutputLineDTO::Stdout(stdout)) => {
                print_stdout_line(sink, stdout)
            }
            ExternalEventsDTO::OutputLine(OutputLineDTO::Stderr(stderr)) => {
                print_stderr_line(sink, stderr)
            }
        }
    }
}

pub fn print_servers_changed<W>(
    w: &mut W,
    servers_started: &ServersChangedDTO,
) -> anyhow::Result<()>
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
            ServerIdentityDTO::Port { .. } => {
                writeln!(w, "[server] http://{}", &server_dto.socket_addr)?
            }
            ServerIdentityDTO::PortNamed { name, .. } => {
                writeln!(w, "[server] [{}] http://{}", name, &server_dto.socket_addr)?
            }
        }
    }
    Ok(())
}

pub fn print_stopped_watching<W: Write>(w: &mut W, evt: &StoppedWatchingDTO) -> anyhow::Result<()> {
    for x in &evt.paths {
        writeln!(w, "[watching:stopped] {}", x)?;
    }
    Ok(())
}

pub fn print_input_accepted<W: Write>(w: &mut W, evt: &InputAcceptedDTO) -> anyhow::Result<()> {
    writeln!(w, "[input] {}", evt.path)?;
    Ok(())
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

pub fn print_input_file_changed<W: Write>(w: &mut W, evt: &FileChangedDTO) -> anyhow::Result<()> {
    writeln!(w, "[change:input] {}", evt.path)?;
    Ok(())
}

pub fn print_stdout_line<W: Write>(w: &mut W, line: &StdoutLineDTO) -> anyhow::Result<()> {
    match &line.prefix {
        None => writeln!(w, "{}", line.line)?,
        Some(prefix) => writeln!(w, "\x1b[2m{}\x1b[0m {}", prefix, line.line)?,
    }
    Ok(())
}

pub fn print_stderr_line<W: Write>(w: &mut W, line: &StderrLineDTO) -> anyhow::Result<()> {
    match &line.prefix {
        None => writeln!(w, "{}", line.line)?,
        Some(prefix) => writeln!(w, "\x1b[1;31m{}\x1b[0m {}", prefix, line.line)?,
    }
    Ok(())
}

pub fn short_file_list<A: AsRef<str>>(paths: &[A]) -> String {
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

pub fn print_watching<W: Write>(w: &mut W, evt: &WatchingDTO) -> anyhow::Result<()> {
    for x in &evt.paths {
        writeln!(w, "[watching {}] {}", evt.debounce, x)?;
    }
    Ok(())
}
