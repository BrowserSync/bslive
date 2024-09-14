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

#[cfg(not(target_os = "windows"))]
fn platform_accepts(evt: &notify::Event) -> bool {
    match evt.kind {
        EventKind::Any => false,
        EventKind::Access(..) => false,
        EventKind::Create(..) => false,
        EventKind::Modify(modify) => match modify {
            #[allow(clippy::match_like_matches_macro)]
            ModifyKind::Data(data) => match data {
                DataChange::Content => true,
                _ => false,
            },
            #[allow(clippy::match_like_matches_macro)]
            ModifyKind::Metadata(meta) => match meta {
                MetadataKind::Any => true,
                _ => false,
            },
            ModifyKind::Name(..) => false,
            ModifyKind::Other => false,
            ModifyKind::Any => false,
        },
        EventKind::Remove(..) => false,
        EventKind::Other => false,
    }
}

#[cfg(target_os = "windows")]
fn platform_accepts(evt: &notify::Event) -> bool {
    match evt.kind {
        EventKind::Any => false,
        EventKind::Access(..) => false,
        EventKind::Create(..) => false,
        EventKind::Modify(modify) => match modify {
            ModifyKind::Any => true,
            ModifyKind::Data(data) => match data {
                DataChange::Content => true,
                _ => false,
            },
            ModifyKind::Metadata(meta) => match meta {
                MetadataKind::Any => true,
                _ => false,
            },
            ModifyKind::Name(..) => false,
            ModifyKind::Other => false,
        },
        EventKind::Remove(..) => false,
        EventKind::Other => false,
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
    return rel
        .map(|p| match p.components().next() {
            None => false,
            Some(Component::Normal(str)) => excluded.contains(str),
            Some(Component::Prefix(_)) => unreachable!("here? Prefix"),
            Some(Component::RootDir) => unreachable!("here? RootDir"),
            Some(Component::CurDir) => unreachable!("here? CurDir"),
            Some(Component::ParentDir) => unreachable!("here? ParentDir"),
        })
        .unwrap_or(false);
}

#[test]
fn matches_single_glob() {
    use glob_match::glob_match;
    let p1 = "public/css/style.css";
    let g = "public/css/*.css";
    let actual = glob_match(g, p1);
    assert_eq!(actual, true);
}
#[test]
fn matches_relative_glob_2() {
    use glob_match::glob_match;
    let relative_path = [
        (
            "/qwe/qwe/qwe/qw/ebuild/app-debug/debugger/debugger.css",
            true,
        ),
        ("/qwe/qwe/qwe/qw/ebuild/app-debug/oops.txt", true),
    ];
    let glob = "**/*.{css,txt}";
    for (path, expected) in relative_path {
        let actual = glob_match(glob, path);
        assert_eq!(actual, expected);
    }
}

#[test]
fn matches_absolute_glob() {
    use glob_match::glob_match;

    let test_cases = vec![
        (
            "/absolute/path/public/css/style.css",
            "/absolute/path/public/css/*.css",
            true,
        ),
        (
            "/absolute/path/public/js/script.js",
            "/absolute/path/public/css/*.css",
            false,
        ),
        (
            "/absolute/path/public/css/nested/style.css",
            "/absolute/path/public/css/**/*.css",
            true,
        ),
        (
            "/absolute/path/public/js/nested/script.js",
            "/absolute/path/public/css/**/*.css",
            false,
        ),
        (
            "/users/documents/reports/january/cash_flow.xls",
            "/users/documents/reports/**/*.xls",
            true,
        ),
        (
            "/users/documents/reports/february/income.txt",
            "/users/documents/reports/**/*.xls",
            false,
        ),
        (
            "/users/documents/reports/yearly/cash_flow.xls",
            "/users/documents/reports/**/*.xls",
            true,
        ),
        (
            "/users/images/vacation/summer/pic.jpg",
            "/users/images/vacation/**/*.jpg",
            true,
        ),
        (
            "/users/images/vacation/winter/snow.png",
            "/users/images/vacation/**/*.jpg",
            false,
        ),
    ];

    for (path, glob, expected) in test_cases {
        let actual = glob_match(glob, path);
        assert_eq!(actual, expected);
    }
}
