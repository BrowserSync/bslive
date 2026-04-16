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
                        || is_excluded_postfix(&p.as_path())
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
        accept,
        platform,
        branch,
        event_kind = ?evt.kind,
    );
}

/// macOS (FSEvents) vs other Unix (inotify, etc.): almost the same rules; Linux additionally accepts
/// [`DataChange::Any`] so data writes match macOS forwarding. Separate `platform` labels in
/// [`trace_platform_decision`] keep audit logs comparable.
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
                    // Linux/inotify often reports writes as `Data(Any)`; macOS/FSEvents usually uses
                    // `Data(Content)`. Accept `Any` on Linux only so forwards match macOS for the same UX.
                    DataChange::Any => (cfg!(target_os = "linux"), "Modify::Data::Any"),
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
    #[test]
    fn test_postfix() {
        let change = Path::new("/Users/shaneosbourne/.index.html.swp");
        let excluded = is_excluded_postfix(&change);
        assert_eq!(excluded, true);
    }
    #[test]
    fn test_vim_4913() {
        let p1 = Path::new("/some/path/4913");
        assert_eq!(is_ignored_path_type(&p1), true);
        let p2 = Path::new("/some/path/5036");
        assert_eq!(is_ignored_path_type(&p2), true);
        let p3 = Path::new("/some/path/6020");
        assert_eq!(is_ignored_path_type(&p3), true);
        let p4 = Path::new("/some/path/4912");
        assert_eq!(is_ignored_path_type(&p4), false);
        let p5 = Path::new("/some/path/not_a_number");
        assert_eq!(is_ignored_path_type(&p5), false);
        let p6 = Path::new("/some/path/4913.txt");
        assert_eq!(is_ignored_path_type(&p6), false);
        let p7 = Path::new("/some/path/6143"); // 11th value in sequence
        assert_eq!(is_ignored_path_type(&p7), false);
    }
}

fn is_ignored_path_type<P: AsRef<Path>>(subject: &P) -> bool {
    let path_ref = subject.as_ref();
    let encoded = path_ref.as_os_str().as_encoded_bytes();

    // vim backup stuff
    if encoded.ends_with(b"~") {
        return true;
    }

    // vim 4913 test files (used on linux/unix to check directory permissions)
    // Starts at 4913 and increments by 123 if it already exists.
    // We only check for the first 10 increments as it's highly unlikely
    // that Vim will cycle through more than that in practice.
    let bytes = path_ref.file_name().map(|name| name.as_encoded_bytes());
    matches!(
        bytes,
        Some(
            b"4913"
                | b"5036"
                | b"5159"
                | b"5282"
                | b"5405"
                | b"5528"
                | b"5651"
                | b"5774"
                | b"5897"
                | b"6020",
        )
    )
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

fn is_excluded_postfix<P: AsRef<Path>>(subject: &P) -> bool {
    let path_ref = subject.as_ref();
    match path_ref.extension().map(|x| x.as_encoded_bytes()) {
        // vim swap stuff
        Some(b"swp") => true,
        Some(b"swo") => true,
        Some(b"swn") => true,
        _ => false,
    }
}
