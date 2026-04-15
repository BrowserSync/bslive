use crate::archy::{archy, overlay_results, ArchyNode, Prefix};
use crate::{
    FileChangedDTO, FilesChangedDTO, InputAcceptedDTO, OutputLineDTO, ServerIdentityDTO,
    ServersChangedDTO, StderrLineDTO, StdoutLineDTO, StoppedWatchingDTO, WatchingDTO,
};
use bsnext_output::OutputWriterTrait;
use bsnext_task::task_report::TaskReport;
use bsnext_task::NodePath;
use std::collections::HashMap;
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
    TaskAction(TaskActionDTO),
    TaskTreePreview(TaskTreePreview),
    TaskTreeSummary(TaskTreeSummary),
}

#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskActionDTO {
    pub stage: TaskActionStageDTO,
}

#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskTreePreview {
    pub tree: ArchyNode,
    pub will_exec: bool,
}

#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskTreeSummary {
    pub tree: ArchyNode,
    pub report_map: HashMap<String, TaskReportDTO>,
}

impl TaskTreeSummary {
    pub fn from_report(tree: ArchyNode, report_map: &HashMap<NodePath, TaskReport>) -> Self {
        let report_map_dto = report_map
            .iter()
            .map(|(k, v)| (k.to_string(), TaskReportDTO::from(v.clone())))
            .collect();
        Self {
            tree,
            report_map: report_map_dto,
        }
    }
}

/// @discriminator kind
#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum TaskActionStageDTO {
    Started {
        tree: ArchyNode,
    },
    Ended {
        tree: ArchyNode,
        report: TaskReportDTO,
        report_map: HashMap<String, TaskReportDTO>,
    },
    Error,
}

#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskReportDTO {
    pub result: TaskResultDTO,
}

#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
pub struct TaskResultDTO {
    #[allow(dead_code)]
    pub conclusion: TaskConclusionDTO,
    #[allow(dead_code)]
    pub invocation_id: InvocationIdDTO,
    #[allow(dead_code)]
    pub task_reports: Vec<TaskReportDTO>,
}

#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "kind", content = "payload")]
pub enum TaskConclusionDTO {
    Ok,
    Err(String),
    Cancelled,
}

#[typeshare]
#[derive(Debug, Clone, serde::Serialize)]
pub struct InvocationIdDTO(pub String);

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
            ExternalEventsDTO::TaskAction(action) => print_task_action(sink, action),
            ExternalEventsDTO::TaskTreePreview(TaskTreePreview {
                tree,
                will_exec: false,
            }) => print_task_tree(sink, tree),
            ExternalEventsDTO::TaskTreePreview(TaskTreePreview {
                tree,
                will_exec: true,
            }) => print_task_tree_preview(sink, tree),
            ExternalEventsDTO::TaskTreeSummary(TaskTreeSummary { tree, report_map }) => {
                print_task_tree_summary(sink, tree, report_map)
            }
        }
    }
}

fn print_task_tree<W: Write>(w: &mut W, tree: &ArchyNode) -> anyhow::Result<()> {
    let s = archy(tree, Prefix::None);
    write!(w, "{s}")?;
    Ok(())
}

fn print_task_tree_preview<W: Write>(w: &mut W, tree: &ArchyNode) -> anyhow::Result<()> {
    let s = archy(tree, Prefix::None);
    write!(w, "{s}")?;
    writeln!(w, "continuing after 2 seconds...")?;
    Ok(())
}

fn print_task_tree_summary<W: Write>(
    w: &mut W,
    tree: &ArchyNode,
    report_map: &HashMap<String, TaskReportDTO>,
) -> anyhow::Result<()> {
    let tree_with_results = overlay_results(tree, report_map);
    let s = archy(&tree_with_results, Prefix::None);
    write!(w, "{s}")?;
    Ok(())
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
        writeln!(w, "[watching:stopped] {x}")?;
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
        Some(prefix) => {
            let color = hash(prefix) % 256;
            writeln!(w, "\x1b[38;5;{}m{}\x1b[0m {}", color, prefix, line.line)?
        }
    }
    Ok(())
}

fn hash(s: &str) -> u32 {
    s.bytes()
        .fold(0u32, |acc, b| acc.wrapping_add(b as u32).wrapping_mul(31))
}

pub fn print_stderr_line<W: Write>(w: &mut W, line: &StderrLineDTO) -> anyhow::Result<()> {
    match &line.prefix {
        None => writeln!(w, "{}", line.line)?,
        Some(prefix) => writeln!(w, "\x1b[1;31m{}\x1b[0m {}", prefix, line.line)?,
    }
    Ok(())
}

pub fn print_task_action<W: Write>(w: &mut W, action_dto: &TaskActionDTO) -> anyhow::Result<()> {
    // let id = action_dto.id;
    match &action_dto.stage {
        TaskActionStageDTO::Started { tree: _ } => {
            // configure if we announce starts?
            // let s = archy(tree, Prefix::None);
            // write!(w, "{s}")?;
        }
        TaskActionStageDTO::Ended {
            report,
            tree,
            report_map,
        } => match report.result.conclusion {
            TaskConclusionDTO::Ok => {}
            TaskConclusionDTO::Err(_) => {
                let tree_with_results = overlay_results(tree, report_map);
                let s = archy(&tree_with_results, Prefix::None);
                write!(w, "{s}")?;
            }
            TaskConclusionDTO::Cancelled => {}
        },
        TaskActionStageDTO::Error => {}
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
