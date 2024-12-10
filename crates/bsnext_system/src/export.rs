use bsnext_core::export::ExportCommand;
use bsnext_core::runtime_ctx::RuntimeCtx;
use bsnext_core::server::router::common::{into_state, uri_to_res_parts};
use bsnext_fs_helpers::WriteMode;
use bsnext_input::playground::Playground;
use bsnext_input::route::{Route, RouteKind};
use bsnext_input::server_config::ServerConfig;
use futures_util::future::join_all;
use http::response::Parts;
use std::clone::Clone;
use std::path::PathBuf;
use std::time::Duration;

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

pub async fn test_playground_export(cwd: &PathBuf, cmd: &ExportCommand) -> anyhow::Result<()> {
    let pg = Playground {
        html: "hello world".to_string(),
        ..Default::default()
    };

    let server = ServerConfig {
        playground: Some(pg),
        ..Default::default()
    };
    let routes = server.combined_routes();
    let state = into_state(server);

    let raw_entries = routes.iter().filter_map(only_raw_entries);
    let raw_entry_paths = raw_entries.clone().map(|r| r.filepath);
    let async_requests = raw_entries.map(|req| uri_to_res_parts(state.clone(), req.url_path));

    let ctx = RuntimeCtx::new(cwd);

    let job_results = join_all(async_requests)
        .await
        .into_iter()
        .zip(raw_entry_paths)
        .map(to_export_type);

    let sink = match cmd.dry_run {
        true => print_sink,
        false => fs_sink,
    };

    let sinked = job_results
        .map(|ref ex_type| sink(ex_type, &ctx))
        .collect::<Vec<_>>();

    dbg!(sinked);

    Ok(())
}

fn fs_sink(ex_type: &ExportType, ctx: &RuntimeCtx) -> anyhow::Result<()> {
    match ex_type {
        ExportType::Write {
            export_result,
            filepath,
        } => {
            write_one(export_result, filepath, ctx)?;
        }
        ExportType::Excluded { reason: _ } => {
            // nothing
        }
    }
    Ok(())
}

fn print_sink(ex_type: &ExportType, ctx: &RuntimeCtx) -> anyhow::Result<()> {
    match ex_type {
        ExportType::Write {
            export_result,
            filepath,
        } => {
            println!(
                "will write {} bytes to {}",
                export_result.1.len(),
                ctx.cwd().join(filepath).display()
            );
        }
        ExportType::Excluded { reason } => {
            println!("Ignoring {:?}", reason)
        }
    }
    Ok(())
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
    filepath: &PathBuf,
    ctx: &RuntimeCtx,
) -> anyhow::Result<PathBuf> {
    let next = ctx.cwd().join("test01").join(filepath);
    let dir = next.parent();
    if let Some(dir) = dir {
        bsnext_fs_helpers::create_dir(dir, &WriteMode::Override)?;
    }
    let (_, ref body, _) = export_result;
    Ok(bsnext_fs_helpers::fs_write_input_src(
        ctx.cwd(),
        &next,
        body,
        &WriteMode::Override,
    )?)
}
