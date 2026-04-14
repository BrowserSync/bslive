use crate::inner_fs_event_handler::InnerChangeEvent;
use notify::event::{DataChange, MetadataKind, ModifyKind};
use notify::EventKind;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Component, Path};
use std::sync::Arc;
use tokio::sync::broadcast;

pub fn create_watcher(
    sender: Arc<broadcast::Sender<InnerChangeEvent>>,
    cwd: &Path,
) -> notify::Result<notify::RecommendedWatcher> {
    let cwd_c = cwd.to_owned();
    notify::recommended_watcher(move |res: Result<notify::Event, _>| {
        let span = tracing::span!(tracing::Level::TRACE, "raw");
        let _guard = span.enter();
        match res {
            // todo: fix this, caused by log output on linux
            // 2026-02-15T11:23:28.459104Z TRACE notify-rs inotify loop raw: [not-accepted] event=Event { kind: Modify(Data(Any)), paths: ["/usr/local/ddg/www-release/frontend/./bslive.log"], attr:tracker: None, attr:flag: None, attr:info: None, attr:source: None }
            Ok(event) if platform_accepts(&event) => {
                if event.paths.iter().any(|p| {
                    is_ignored_path_type(&p.as_path())
                        || is_auto_excluded(&cwd_c.as_path(), &p.as_path())
                }) {
                    tracing::trace!(?event.paths, "[ignored]");
                    return;
                }
                let msg = InnerChangeEvent {
                    absolute_path: event.paths.first().unwrap().into(),
                };
                tracing::trace!("  └ [accept] {:?}", event);
                match sender.send(msg) {
                    Ok(_) => {}
                    Err(e) => tracing::error!(?e),
                }
            }
            Ok(event) => {
                tracing::trace!(?event, "[not-accepted]")
            }
            Err(e) => {
                tracing::error!("fswadtcher {:?}", e);
            }
        }
    })
}

/// Stable tracing target so cross-platform audits can filter with
/// `RUST_LOG=bsnext_fs::platform_accepts=trace` (or `bsnext_fs=trace`).
fn trace_platform_decision(
    platform: &'static str,
    branch: &'static str,
    evt: &notify::Event,
    accept: bool,
) {
    tracing::trace!(
        target: "bsnext_fs::platform_accepts",
        platform,
        branch,
        event_kind = ?evt.kind,
        accept,
    );
}

/// macOS (FSEvents) vs other Unix (e.g. inotify): same acceptance rules for now, separate
/// `platform` labels in [`trace_platform_decision`] so audit logs are comparable per OS.
#[cfg(not(target_os = "windows"))]
fn platform_accepts(evt: &notify::Event) -> bool {
    #[cfg(target_os = "macos")]
    const PLATFORM: &str = "macos";
    #[cfg(not(target_os = "macos"))]
    const PLATFORM: &str = "unix";
    platform_accepts_posix(evt, PLATFORM)
}

#[cfg(not(target_os = "windows"))]
fn platform_accepts_posix(evt: &notify::Event, platform: &'static str) -> bool {
    match evt.kind {
        EventKind::Any => {
            trace_platform_decision(platform, "Any", evt, false);
            false
        }
        EventKind::Access(kind) => {
            trace_platform_decision(platform, "Access", evt, false);
            let _ = kind;
            false
        }
        EventKind::Create(kind) => {
            trace_platform_decision(platform, "Create", evt, false);
            let _ = kind;
            false
        }
        EventKind::Modify(modify) => match modify {
            ModifyKind::Any => {
                trace_platform_decision(platform, "Modify::Any", evt, false);
                false
            }
            ModifyKind::Data(data) => {
                let (accept, branch) = match data {
                    DataChange::Content => (true, "Modify::Data::Content"),
                    DataChange::Any => (false, "Modify::Data::Any"),
                    DataChange::Size => (false, "Modify::Data::Size"),
                    DataChange::Other => (false, "Modify::Data::Other"),
                };
                trace_platform_decision(platform, branch, evt, accept);
                accept
            }
            ModifyKind::Metadata(meta) => {
                let (accept, branch) = match meta {
                    MetadataKind::Any => (true, "Modify::Metadata::Any"),
                    MetadataKind::AccessTime => (false, "Modify::Metadata::AccessTime"),
                    MetadataKind::WriteTime => (false, "Modify::Metadata::WriteTime"),
                    MetadataKind::Permissions => (false, "Modify::Metadata::Permissions"),
                    MetadataKind::Ownership => (false, "Modify::Metadata::Ownership"),
                    MetadataKind::Extended => (false, "Modify::Metadata::Extended"),
                    MetadataKind::Other => (false, "Modify::Metadata::Other"),
                };
                trace_platform_decision(platform, branch, evt, accept);
                accept
            }
            ModifyKind::Name(mode) => {
                trace_platform_decision(platform, "Modify::Name", evt, false);
                let _ = mode;
                false
            }
            ModifyKind::Other => {
                trace_platform_decision(platform, "Modify::Other", evt, false);
                false
            }
        },
        EventKind::Remove(kind) => {
            trace_platform_decision(platform, "Remove", evt, false);
            let _ = kind;
            false
        }
        EventKind::Other => {
            trace_platform_decision(platform, "EventKind::Other", evt, false);
            false
        }
    }
}

#[cfg(target_os = "windows")]
fn platform_accepts(evt: &notify::Event) -> bool {
    const PLATFORM: &str = "windows";
    match evt.kind {
        EventKind::Any => {
            trace_platform_decision(PLATFORM, "Any", evt, false);
            false
        }
        EventKind::Access(kind) => {
            trace_platform_decision(PLATFORM, "Access", evt, false);
            let _ = kind;
            false
        }
        EventKind::Create(kind) => {
            trace_platform_decision(PLATFORM, "Create", evt, false);
            let _ = kind;
            false
        }
        EventKind::Modify(modify) => match modify {
            ModifyKind::Any => {
                trace_platform_decision(PLATFORM, "Modify::Any", evt, true);
                true
            }
            ModifyKind::Data(data) => {
                let (accept, branch) = match data {
                    DataChange::Content => (true, "Modify::Data::Content"),
                    DataChange::Any => (false, "Modify::Data::Any"),
                    DataChange::Size => (false, "Modify::Data::Size"),
                    DataChange::Other => (false, "Modify::Data::Other"),
                };
                trace_platform_decision(PLATFORM, branch, evt, accept);
                accept
            }
            ModifyKind::Metadata(meta) => {
                let (accept, branch) = match meta {
                    MetadataKind::Any => (true, "Modify::Metadata::Any"),
                    MetadataKind::AccessTime => (false, "Modify::Metadata::AccessTime"),
                    MetadataKind::WriteTime => (false, "Modify::Metadata::WriteTime"),
                    MetadataKind::Permissions => (false, "Modify::Metadata::Permissions"),
                    MetadataKind::Ownership => (false, "Modify::Metadata::Ownership"),
                    MetadataKind::Extended => (false, "Modify::Metadata::Extended"),
                    MetadataKind::Other => (false, "Modify::Metadata::Other"),
                };
                trace_platform_decision(PLATFORM, branch, evt, accept);
                accept
            }
            ModifyKind::Name(mode) => {
                trace_platform_decision(PLATFORM, "Modify::Name", evt, false);
                let _ = mode;
                false
            }
            ModifyKind::Other => {
                trace_platform_decision(PLATFORM, "Modify::Other", evt, false);
                false
            }
        },
        EventKind::Remove(kind) => {
            trace_platform_decision(PLATFORM, "Remove", evt, false);
            let _ = kind;
            false
        }
        EventKind::Other => {
            trace_platform_decision(PLATFORM, "EventKind::Other", evt, false);
            false
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test() {
        let cwd = Path::new("/Users/shaneosbourne/WebstormProjects/browsersync.github.io");
        let change = Path::new("/Users/shaneosbourne/WebstormProjects/browsersync.github.io/node_modules/ajv/dist/vocabularies/applicator/if.d.ts");
        let excluded = is_auto_excluded(&cwd, &change);
        assert_eq!(excluded, true);
    }
}

fn is_ignored_path_type<P: AsRef<Path>>(subject: &P) -> bool {
    subject
        .as_ref()
        .as_os_str()
        .as_encoded_bytes()
        .ends_with(b"~")
}

// todo: If a folder is explicitly watched, these rules should be ignored
fn is_auto_excluded<P: AsRef<Path>>(cwd: &P, subject: &P) -> bool {
    // todo: allow more config here...
    let excluded: HashSet<&OsStr> = [
        "node_modules",
        ".git",
        ".husky",
        ".vscode",
        ".idea",
        ".sass-cache",
        "bslive.log",
    ]
    .into_iter()
    .map(OsStr::new)
    .collect();
    let rel = subject.as_ref().strip_prefix(cwd.as_ref());
    rel.map(|p| match p.components().next() {
        None => false,
        Some(Component::Normal(str)) => excluded.contains(str),
        Some(Component::Prefix(_)) => unreachable!("here? Prefix"),
        Some(Component::RootDir) => unreachable!("here? RootDir"),
        Some(Component::CurDir) => unreachable!("here? CurDir"),
        Some(Component::ParentDir) => unreachable!("here? ParentDir"),
    })
    .unwrap_or(false)
}
