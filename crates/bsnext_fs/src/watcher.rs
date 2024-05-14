use crate::inner_fs_event_handler::InnerChangeEvent;
use notify::event::{
    AccessKind, AccessMode, CreateKind, DataChange, MetadataKind, ModifyKind, RemoveKind,
    RenameMode,
};
use notify::EventKind;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::path::{Component, Path};
use std::sync::Arc;
use tokio::sync::broadcast;

pub fn create_watcher(
    sender: Arc<broadcast::Sender<InnerChangeEvent>>,
    cwd: &Path,
) -> notify::Result<notify::FsEventWatcher> {
    let cwd_c = cwd.to_owned();
    notify::recommended_watcher(move |res: Result<notify::Event, _>| {
        let span = tracing::span!(tracing::Level::TRACE, "raw");
        let _guard = span.enter();
        match res {
            Ok(event) => {
                if event.paths.iter().any(|p| {
                    is_ignored_path_type(&p.as_path())
                        || is_auto_excluded(&cwd_c.as_path(), &p.as_path())
                }) {
                    tracing::trace!(?event.paths, "ignored!!!");
                    return;
                }

                let msg = InnerChangeEvent {
                    absolute_path: event.paths.first().unwrap().into(),
                };
                match event.kind {
                    EventKind::Any => {}
                    EventKind::Access(ac) => match ac {
                        AccessKind::Any => tracing::trace!("EventKind::Access AccessKind::Any"),
                        AccessKind::Read => {
                            tracing::trace!("EventKind::Access AccessKind::Read")
                        }
                        AccessKind::Open(o) => match o {
                            AccessMode::Any => {
                                tracing::trace!(
                                    "EventKind::Access AccessKind::Open AccessMode::Any"
                                )
                            }
                            AccessMode::Execute => {
                                tracing::trace!(
                                    "EventKind::Access AccessKind::Open AccessMode::Execute"
                                )
                            }
                            AccessMode::Read => {
                                tracing::trace!(
                                    "EventKind::Access AccessKind::Open AccessMode::Read"
                                )
                            }
                            AccessMode::Write => {
                                tracing::trace!(
                                    "EventKind::Access AccessKind::Open AccessMode::Write"
                                )
                            }
                            AccessMode::Other => {
                                tracing::trace!(
                                    "EventKind::Access AccessKind::Open AccessMode::Other"
                                )
                            }
                        },
                        AccessKind::Close(c) => match c {
                            AccessMode::Any => {
                                tracing::trace!(
                                    "EventKind::Access AccessKind::Close AccessMode::Any"
                                )
                            }
                            AccessMode::Execute => {
                                tracing::trace!(
                                    "EventKind::Access AccessKind::Close AccessMode::Execute"
                                )
                            }
                            AccessMode::Read => {
                                tracing::trace!(
                                    "EventKind::Access AccessKind::Close AccessMode::Read"
                                )
                            }
                            AccessMode::Write => {
                                tracing::trace!(
                                    "EventKind::Access AccessKind::Close AccessMode::Write"
                                )
                            }
                            AccessMode::Other => {
                                tracing::trace!(
                                    "EventKind::Access AccessKind::Close AccessMode::Other"
                                )
                            }
                        },
                        AccessKind::Other => {
                            tracing::trace!("EventKind::Access AccessKind::Other")
                        }
                    },
                    EventKind::Create(c) => match c {
                        CreateKind::Any => tracing::trace!("EventKind::Create CreateKind::Any"),
                        CreateKind::File => {
                            tracing::trace!("EventKind::Create CreateKind::File")
                        }
                        CreateKind::Folder => {
                            tracing::trace!("EventKind::Create CreateKind::Folder")
                        }
                        CreateKind::Other => {
                            tracing::trace!("EventKind::Create CreateKind::Other")
                        }
                    },
                    EventKind::Modify(modify) => match modify {
                        ModifyKind::Any => tracing::trace!("EventKind::Modify ModifyKind::Any"),
                        ModifyKind::Data(data) => match data {
                            DataChange::Any => {
                                tracing::trace!(
                                    "EventKind::Modify ModifyKind::Data DataChange::Any"
                                );
                            }
                            DataChange::Size => {
                                tracing::trace!(
                                    "EventKind::Modify ModifyKind::Data DataChange::Size"
                                );
                            }
                            DataChange::Content => {
                                tracing::trace!(
                                    "EventKind::Modify ModifyKind::Data DataChange::Content"
                                );
                                match sender.send(msg) {
                                    Ok(_) => {}
                                    Err(e) => tracing::error!(?e),
                                };
                            }
                            DataChange::Other => {
                                tracing::trace!(
                                    "EventKind::Modify ModifyKind::Data DataChange::Other"
                                );
                            }
                        },
                        ModifyKind::Metadata(meta) => {
                            match meta {
                                MetadataKind::Any => {
                                    tracing::trace!(
                                        "EventKind::Modify ModifyKind::Metadata MetadataKind::Any"
                                    );
                                    match sender.send(msg) {
                                        Ok(_) => {}
                                        Err(e) => tracing::error!(?e),
                                    };
                                }
                                MetadataKind::AccessTime => {
                                    tracing::trace!("EventKind::Modify ModifyKind::Metadata MetadataKind::AccessTime")
                                }
                                MetadataKind::WriteTime => {
                                    tracing::trace!("EventKind::Modify ModifyKind::Metadata MetadataKind::WriteTime")
                                }
                                MetadataKind::Permissions => {
                                    tracing::trace!("EventKind::Modify ModifyKind::Metadata MetadataKind::Permissions")
                                }
                                MetadataKind::Ownership => {
                                    tracing::trace!("EventKind::Modify ModifyKind::Metadata MetadataKind::Ownership")
                                }
                                MetadataKind::Extended => {
                                    tracing::trace!("EventKind::Modify ModifyKind::Metadata MetadataKind::Extended")
                                }
                                MetadataKind::Other => {
                                    tracing::trace!("EventKind::Modify ModifyKind::Metadata MetadataKind::Other")
                                }
                            }
                        }
                        ModifyKind::Name(mode) => match mode {
                            RenameMode::Any => {
                                tracing::trace!(
                                    "EventKind::Modify ModifyKind::Name RenameMode::Any"
                                )
                            }
                            RenameMode::To => {
                                tracing::trace!("EventKind::Modify ModifyKind::Name RenameMode::To")
                            }
                            RenameMode::From => {
                                tracing::trace!(
                                    "EventKind::Modify ModifyKind::Name RenameMode::From"
                                )
                            }
                            RenameMode::Both => {
                                tracing::trace!(
                                    "EventKind::Modify ModifyKind::Name RenameMode::Both"
                                )
                            }
                            RenameMode::Other => {
                                tracing::trace!(
                                    "EventKind::Modify ModifyKind::Name RenameMode::Other"
                                )
                            }
                        },
                        ModifyKind::Other => {
                            tracing::trace!("EventKind::Modify ModifyKind::Other")
                        }
                    },
                    EventKind::Remove(remove) => match remove {
                        RemoveKind::Any => tracing::trace!("EventKind::Remove RemoveKind::Any"),
                        RemoveKind::File => {
                            tracing::trace!("EventKind::Remove RemoveKind::File")
                        }
                        RemoveKind::Folder => {
                            tracing::trace!("EventKind::Remove RemoveKind::Folder")
                        }
                        RemoveKind::Other => {
                            tracing::trace!("EventKind::Remove RemoveKind::Other")
                        }
                    },
                    EventKind::Other => tracing::trace!("EventKind::Other"),
                }
            }
            Err(e) => {
                tracing::error!("fswadtcher {:?}", e);
            }
        }
    })
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
