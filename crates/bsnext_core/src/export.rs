use crate::runtime_ctx::RuntimeCtx;
use crate::server::router::common::{into_state, uri_to_res_parts};
use bsnext_fs_helpers::{FsWriteError, WriteMode};
use bsnext_input::route::{Route, RouteKind};
use bsnext_input::server_config::ServerConfig;
use bsnext_output::OutputWriterTrait;
use futures_util::future::join_all;
use http::response::Parts;
use std::clone::Clone;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, serde::Serialize)]
pub enum ExportEvent {
    DryRunDirCreate(PathBuf),
    DryRunFileCreate(PathBuf),
    DidCreateFile(PathBuf),
    DidCreateDir(PathBuf),
}

#[derive(Debug, thiserror::Error)]
pub enum ExportError {
    #[error("{0}")]
    Fs(FsWriteError),
}

impl OutputWriterTrait for ExportError {
    fn write_json<W: Write>(&self, _sink: &mut W) -> anyhow::Result<()> {
        let error_string = self.to_string();
        let named = match self {
            ExportError::Fs(_) => "fs",
        };
        let v = serde_json::json!({
            "kind": named,
            "error": error_string
        });
        writeln!(_sink, "{}", v)?;
        Ok(())
    }

    fn write_pretty<W: Write>(&self, sink: &mut W) -> anyhow::Result<()> {
        match self {
            ExportError::Fs(fs_write_error) => {
                writeln!(sink, "[export]: Error! {}", fs_write_error)?;
            }
        }
        Ok(())
    }
}

impl OutputWriterTrait for ExportEvent {
    fn write_json<W: Write>(&self, _sink: &mut W) -> anyhow::Result<()> {
        let str = serde_json::to_string(&self)?;
        writeln!(_sink, "{}", str)?;
        Ok(())
    }

    fn write_pretty<W: Write>(&self, sink: &mut W) -> anyhow::Result<()> {
        match self {
            ExportEvent::DryRunDirCreate(dir) => {
                writeln!(sink, "[export:dry-run]: would create dir {}", dir.display())?;
            }
            ExportEvent::DryRunFileCreate(file) => writeln!(
                sink,
                "[export:dry-run]: would create file {}",
                file.display()
            )?,
            ExportEvent::DidCreateFile(dir) => {
                writeln!(sink, "[export]: did create file {}", dir.display())?;
            }
            ExportEvent::DidCreateDir(file) => {
                writeln!(sink, "[export]: did create dir {}", file.display())?;
            }
        }
        Ok(())
    }
}

#[derive(Debug)]
enum ExportType {
    Write {
        export_result: ExportResult,
        filepath: PathBuf,
    },
    Excluded {
        reason: ExcludeReason,
    },
}

#[derive(Debug)]
enum ExcludeReason {
    BadRequest,
}

#[derive(Debug)]
struct ExportRequest<'a> {
    pub url_path: &'a str,
    pub filepath: PathBuf,
}

type ExportResult = (Parts, String, Duration);

#[derive(Debug, Clone, clap::Parser)]
pub struct ExportCommand {
    /// The folder to export the files to. For current, provide '.'
    #[arg(long = "dir")]
    pub out_dir: PathBuf,
    /// When provided, just prints what might happen instead of actually causing side effects
    #[arg(long)]
    pub dry_run: bool,

    /// Paths to serve + possibly watch, incompatible with `-i` option
    pub paths: Vec<String>,
}

pub async fn export_one_server(
    cwd: &PathBuf,
    server: ServerConfig,
    cmd: &ExportCommand,
    write_mode: WriteMode,
) -> Result<Vec<ExportEvent>, ExportError> {
    let routes = server.combined_routes();
    let state = into_state(server);

    let raw_entries = routes.iter().filter_map(only_raw_entries);
    let raw_entry_paths = raw_entries.clone().map(|r| r.filepath);
    let async_requests = raw_entries.map(|req| uri_to_res_parts(state.clone(), req.url_path));

    let ctx = RuntimeCtx::new(cwd);
    // let job_count = raw_entry_paths.len();

    join_all(async_requests)
        .await
        .into_iter()
        .zip(raw_entry_paths)
        .map(to_export_type)
        .try_fold(vec![], move |mut acc, ref ex_type| match cmd.dry_run {
            true => match print_sink(ex_type, &cmd.out_dir, &ctx) {
                Ok(evts) => {
                    acc.extend(evts);
                    Ok(acc)
                }
                Err(e) => Err(e),
            },
            false => match fs_sink(ex_type, &cmd.out_dir, &ctx, &write_mode) {
                Ok(evts) => {
                    acc.extend(evts);
                    Ok(acc)
                }
                Err(e) => Err(e),
            },
        })
}

fn fs_sink(
    ex_type: &ExportType,
    out_dir: &PathBuf,
    ctx: &RuntimeCtx,
    write_mode: &WriteMode,
) -> Result<Vec<ExportEvent>, ExportError> {
    match ex_type {
        ExportType::Write {
            export_result,
            filepath,
        } => {
            let filepath = ctx.cwd().join(out_dir).join(filepath);
            write_one(export_result, &filepath, ctx, write_mode).map_err(ExportError::Fs)
        }
        ExportType::Excluded { reason: _ } => Ok(vec![]),
    }
}

fn print_sink(
    ex_type: &ExportType,
    out_dir: &PathBuf,
    ctx: &RuntimeCtx,
) -> Result<Vec<ExportEvent>, ExportError> {
    let mut events = vec![];
    match ex_type {
        ExportType::Write { filepath, .. } => {
            let path = ctx.cwd().join(out_dir).join(filepath);
            events.push(ExportEvent::DryRunFileCreate(path));
        }
        ExportType::Excluded { reason } => {
            todo!("Ignoring {:?}", reason)
        }
    }
    Ok(events)
}

fn to_export_type((export_result, filepath): (ExportResult, PathBuf)) -> ExportType {
    let (parts, _, _) = &export_result;
    if parts.status.as_u16() == 200 {
        ExportType::Write {
            export_result,
            filepath,
        }
    } else {
        ExportType::Excluded {
            reason: ExcludeReason::BadRequest,
        }
    }
}

fn only_raw_entries(route: &Route) -> Option<ExportRequest> {
    match &route.kind {
        RouteKind::Raw(..) => Some(ExportRequest {
            filepath: route.as_filepath(),
            url_path: route.url_path(),
        }),
        _ => None,
    }
}

fn write_one(
    export_result: &ExportResult,
    filepath: &Path,
    ctx: &RuntimeCtx,
    write_mode: &WriteMode,
) -> Result<Vec<ExportEvent>, FsWriteError> {
    let dir = filepath.parent();
    let mut events = vec![];
    if let Some(dir) = dir {
        fs::create_dir_all(dir).map_err(FsWriteError::FailedDir)?;
    }
    let (_, ref body, _) = export_result;
    let pb = bsnext_fs_helpers::fs_write_str(ctx.cwd(), filepath, body, write_mode)?;
    events.push(ExportEvent::DidCreateFile(pb.clone()));
    Ok(events)
}
