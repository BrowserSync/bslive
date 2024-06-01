use crate::BsSystem;
use actix::{Actor, Addr, AsyncContext};
use std::hash::Hash;

use bsnext_core::servers_supervisor::file_changed_handler::{FileChanged, FilesChanged};
use bsnext_dto::{ExternalEvents, StoppedWatching, Watching};
use bsnext_fs::watch_path_handler::RequestWatchPath;
use bsnext_fs::{Debounce, FsEvent, FsEventContext, FsEventKind};
use bsnext_input::route::{
    DebounceDuration, DirRoute, FilterKind, RouteKind, Spec, SpecOpts, WatchOpts,
};
use bsnext_input::server_config::Identity;
use bsnext_input::Input;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bsnext_fs::actor::FsWatcher;

use tracing::{span, trace_span, Level};

#[derive(Debug, Clone)]
pub struct Monitor {
    pub(crate) addr: Addr<FsWatcher>,
    pub(crate) path: PathBuf,
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct MonitorInput {
    pub path: PathBuf,
    pub cwd: PathBuf,
}

impl actix::Handler<MonitorInput> for BsSystem {
    type Result = ();

    fn handle(&mut self, msg: MonitorInput, ctx: &mut Self::Context) -> Self::Result {
        let mut input_watcher = bsnext_fs::actor::FsWatcher::for_input(&msg.cwd, 0);

        // todo: does this need to be configurable (eg: by main config)?
        input_watcher.with_debounce(Debounce::Trailing {
            duration: Duration::from_millis(300),
        });

        tracing::trace!("[main.rs] starting input monitor");

        let input_watcher_addr = input_watcher.start();
        self.input_monitors.push(input_watcher_addr.clone());

        input_watcher_addr.do_send(RequestWatchPath {
            recipients: vec![ctx.address().recipient()],
            path: msg.path.to_path_buf(),
        });
    }
}

impl actix::Handler<FsEvent> for BsSystem {
    type Result = ();

    fn handle(&mut self, msg: FsEvent, _ctx: &mut Self::Context) -> Self::Result {
        match msg.kind {
            FsEventKind::ChangeBuffered(buffer_change) => {
                let span = span!(Level::TRACE, "FsEventKind::ChangeBuffered");
                let _guard = span.enter();
                tracing::debug!(msg.event_count = buffer_change.events.len(), msg.ctx = ?msg.ctx, ?buffer_change);
                // let id = msg.ctx_id();
                let paths = buffer_change
                    .events
                    .iter()
                    .map(|evt| evt.absolute.to_owned())
                    .collect::<Vec<_>>();
                let as_strings = paths
                    .iter()
                    .map(|p| p.to_string_lossy().to_string())
                    .collect::<Vec<String>>();
                if let Some(servers) = &self.servers_addr {
                    servers.do_send(FilesChanged {
                        paths: paths.clone(),
                        id: msg.ctx.id(),
                    })
                }
                let evt =
                    ExternalEvents::FilesChanged(bsnext_dto::FilesChangedDTO { paths: as_strings });
                self.publish_external_event(evt);
            }
            FsEventKind::Change(inner) => {
                let span = trace_span!("FsEventKind::Change", ?inner.absolute_path);
                let _guard = span.enter();
                match msg.ctx {
                    FsEventContext::InputFile { id: _ } => {
                        tracing::info!(?inner, "InputFile file changed");
                        let input = Input::from_input_path(&inner.absolute_path);

                        let Ok(input) = input else {
                            tracing::debug!("ignoring FsWatchEvent because the input was invalid");
                            let err = input.unwrap_err();
                            tracing::error!(?err, "{}", err);
                            return;
                        };

                        tracing::debug!("InputFile file was deserialized");

                        self.accept_input(&input);
                        self.inform_servers(input);

                        let evt = ExternalEvents::InputFileChanged(
                            bsnext_dto::FileChanged::from_path_buf(&inner.path),
                        );
                        self.publish_external_event(evt);
                    }
                    FsEventContext::Other { id } => {
                        tracing::trace!(?inner, "Other file changed");
                        // todo: tie these changed to an input identity?
                        if let Some(servers) = &self.servers_addr {
                            servers.do_send(FileChanged {
                                path: inner.absolute_path.clone(),
                                id,
                            })
                        }
                        let evt = ExternalEvents::FileChanged(
                            bsnext_dto::FileChanged::from_path_buf(&inner.path),
                        );

                        self.publish_external_event(evt);
                    }
                }
            }
            FsEventKind::PathAdded(path) => {
                let span = trace_span!("FsEventKind::PathAdded", ?path);
                let _guard = span.enter();
                let evt =
                    ExternalEvents::Watching(Watching::from_path_buf(&path.path, path.debounce));
                self.publish_external_event(evt);
            }
            FsEventKind::PathRemoved(path) => {
                let span = trace_span!("FsEventKind::PathRemoved", ?path);
                let _guard = span.enter();
                let evt =
                    ExternalEvents::WatchingStopped(StoppedWatching::from_path_buf(&path.path));
                self.publish_external_event(evt);
            }
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum AnyWatchable {
    Server(ServerWatchable),
    Route(RouteWatchable),
}

impl AnyWatchable {
    pub fn spec_opts(&self) -> Option<&SpecOpts> {
        match self {
            AnyWatchable::Server(server) => server.spec.opts.as_ref(),
            AnyWatchable::Route(route) => route.spec.opts.as_ref(),
        }
    }
    pub fn watch_path(&self) -> &Path {
        match self {
            AnyWatchable::Server(server) => &server.dir,
            AnyWatchable::Route(route) => &route.dir,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct ServerWatchable {
    pub server_identity: Identity,
    pub dir: PathBuf,
    pub spec: Spec,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct RouteWatchable {
    pub server_identity: Identity,
    pub route_path: String,
    pub dir: PathBuf,
    pub spec: Spec,
}

pub fn to_server_watchables(input: &Input) -> Vec<ServerWatchable> {
    input
        .servers
        .iter()
        .flat_map(|server_config| {
            server_config.watchers.iter().map(|watcher| {
                // let spec = to_spec(&r.watch_opts);
                ServerWatchable {
                    server_identity: server_config.identity.clone(),
                    dir: PathBuf::from(&watcher.dir),
                    spec: Spec {
                        opts: Some(SpecOpts {
                            debounce: watcher.debounce_ms.map(DebounceDuration::Ms),
                            filter: watcher.filter.clone(),
                        }),
                    },
                }
            })
        })
        .collect()
}

pub fn to_route_watchables(input: &Input) -> Vec<RouteWatchable> {
    input
        .servers
        .iter()
        .flat_map(|server_config| {
            server_config
                .routes
                .iter()
                .filter(|r| r.watch_opts.is_enabled())
                .filter_map(|r| match &r.kind {
                    RouteKind::Html { .. } => None,
                    RouteKind::Json { .. } => None,
                    RouteKind::Raw { .. } => None,
                    RouteKind::Sse { .. } => None,
                    RouteKind::Proxy(_) => None,
                    RouteKind::Dir(DirRoute { dir }) => {
                        let spec = to_spec(&r.watch_opts);
                        Some(RouteWatchable {
                            server_identity: server_config.identity.clone(),
                            route_path: r.path.to_string(),
                            dir: PathBuf::from(dir),
                            spec,
                        })
                    }
                })
        })
        .collect()
}

fn to_spec(wo: &WatchOpts) -> Spec {
    match wo {
        WatchOpts::Bool(enabled) if !*enabled => unreachable!("should be handled..."),
        WatchOpts::Bool(enabled) if *enabled => Spec { opts: None },
        WatchOpts::InlineGlob(glob) => Spec {
            opts: Some(SpecOpts {
                debounce: None,
                filter: Some(FilterKind::Glob {
                    glob: glob.to_string(),
                }),
            }),
        },
        WatchOpts::Spec(spec) => spec.to_owned(),
        WatchOpts::Bool(_) => todo!("unreachable"),
    }
}
