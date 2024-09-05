use crate::BsSystem;
use actix::{Actor, Addr, AsyncContext};
use std::hash::Hash;

use bsnext_core::servers_supervisor::file_changed_handler::{FileChanged, FilesChanged};
use bsnext_dto::{ExternalEvents, StoppedWatching, Watching};
use bsnext_fs::watch_path_handler::RequestWatchPath;
use bsnext_fs::{
    BufferedChangeEvent, ChangeEvent, Debounce, FsEvent, FsEventKind, PathAddedEvent, PathEvent,
};
use bsnext_input::route::{DebounceDuration, DirRoute, FilterKind, RouteKind, Spec, SpecOpts};
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::{Input, InputError, PathDefinition, PathDefs, PathError};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use bsnext_fs::actor::FsWatcher;

use bsnext_input::watch_opts::WatchOpts;
use tracing::trace_span;

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
        let span = trace_span!("monitor_input", ?msg.path, ?msg.cwd);
        let s = Arc::new(span);
        let span_c = s.clone();
        let _guard = s.enter();
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
            span: span_c.clone(),
        });
    }
}

impl BsSystem {
    #[tracing::instrument(skip(self))]
    fn handle_buffered(
        &mut self,
        msg: &FsEvent,
        buf: &BufferedChangeEvent,
    ) -> Option<ExternalEvents> {
        tracing::debug!(msg.event_count = buf.events.len(), msg.ctx = ?msg.ctx, ?buf);
        let paths = buf
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
                ctx: msg.ctx.clone(),
            })
        }
        // todo(alpha): need to exclude changes to the input file if this event has captured it
        Some(ExternalEvents::FilesChanged(bsnext_dto::FilesChangedDTO {
            paths: as_strings,
        }))
    }
    fn handle_change(&mut self, msg: &FsEvent, inner: &ChangeEvent) -> Option<ExternalEvents> {
        let span = trace_span!("handle_change", ?inner.absolute_path);
        let _guard = span.enter();
        match msg.ctx.id() {
            0 => {
                tracing::info!(?inner, "InputFile file changed");
                let input = Input::from_input_path(&inner.absolute_path);

                let Ok(input) = input else {
                    let err = input.unwrap_err();
                    return Some(ExternalEvents::InputError(err.into()));
                };

                self.accept_watchables(&input);
                self.resolve_servers(input);

                Some(ExternalEvents::InputFileChanged(
                    bsnext_dto::FileChanged::from_path_buf(&inner.path),
                ))
            }
            _id => {
                tracing::trace!(?inner, "Other file changed");
                // todo: tie these changed to an input identity?
                if let Some(servers) = &self.servers_addr {
                    servers.do_send(FileChanged {
                        path: inner.absolute_path.clone(),
                        ctx: msg.ctx.clone(),
                    })
                }
                Some(ExternalEvents::FileChanged(
                    bsnext_dto::FileChanged::from_path_buf(&inner.path),
                ))
            }
        }
    }
    #[tracing::instrument(skip(self))]
    fn handle_path_added(&mut self, path: &PathAddedEvent) -> Option<ExternalEvents> {
        Some(ExternalEvents::Watching(Watching::from_path_buf(
            &path.path,
            path.debounce,
        )))
    }

    #[tracing::instrument(skip(self))]
    fn handle_path_removed(&mut self, path: &PathEvent) -> Option<ExternalEvents> {
        Some(ExternalEvents::WatchingStopped(
            StoppedWatching::from_path_buf(&path.path),
        ))
    }

    #[tracing::instrument(skip(self))]
    fn handle_path_not_found(&mut self, pdo: &PathEvent) -> Option<ExternalEvents> {
        let as_str = pdo.path.to_string_lossy().to_string();
        let cwd = self.cwd.clone().unwrap();
        let abs = cwd.join(&as_str);
        let def = PathDefinition {
            input: as_str,
            cwd: self.cwd.clone().unwrap(),
            absolute: abs,
        };
        let e = InputError::PathError(PathError::MissingPaths {
            paths: PathDefs(vec![def]),
        });
        Some(ExternalEvents::InputError(e.into()))
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct OverrideInput {
    pub input: Input,
}

impl actix::Handler<OverrideInput> for BsSystem {
    type Result = ();

    fn handle(&mut self, msg: OverrideInput, _ctx: &mut Self::Context) -> Self::Result {
        self.accept_watchables(&msg.input);
        self.resolve_servers(msg.input);
    }
}

impl actix::Handler<FsEvent> for BsSystem {
    type Result = ();
    #[tracing::instrument(skip(self, _ctx), name = "FsEvent handler for BsSystem", parent=msg.span.as_ref().and_then(|s|s.id()))]
    fn handle(&mut self, msg: FsEvent, _ctx: &mut Self::Context) -> Self::Result {
        let next = match &msg.kind {
            FsEventKind::ChangeBuffered(buffer_change) => self.handle_buffered(&msg, buffer_change),
            FsEventKind::Change(inner) => self.handle_change(&msg, inner),
            FsEventKind::PathAdded(path) => self.handle_path_added(path),
            FsEventKind::PathRemoved(path) => self.handle_path_removed(path),
            FsEventKind::PathNotFoundError(pdo) => self.handle_path_not_found(pdo),
        };
        if let Some(ext) = next {
            self.publish_external_event(ext)
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub enum AnyWatchable {
    Server(ServerWatchable),
    Input(InputWatchable),
    Route(RouteWatchable),
}

impl AnyWatchable {
    pub fn spec_opts(&self) -> Option<&SpecOpts> {
        match self {
            AnyWatchable::Server(server) => server.spec.opts.as_ref(),
            AnyWatchable::Route(route) => route.spec.opts.as_ref(),
            AnyWatchable::Input(_) => todo!("implement input spec opts"),
        }
    }
    pub fn watch_path(&self) -> &Path {
        match self {
            AnyWatchable::Server(server) => &server.dir,
            AnyWatchable::Route(route) => &route.dir,
            AnyWatchable::Input(input) => &input.path,
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct ServerWatchable {
    pub server_identity: ServerIdentity,
    pub dir: PathBuf,
    pub spec: Spec,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct InputWatchable {
    pub path: PathBuf,
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct RouteWatchable {
    pub server_identity: ServerIdentity,
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
                .filter(|r| r.opts.watch.is_enabled())
                .filter_map(|r| match &r.kind {
                    RouteKind::Raw(_) => None,
                    RouteKind::Proxy(_) => None,
                    RouteKind::Dir(DirRoute { dir, .. }) => {
                        let spec = to_spec(&r.opts.watch);
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
