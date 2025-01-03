use crate::{BsSystem, InputMonitor, ResolveServers};
use actix::{Actor, Addr, AsyncContext};
use std::hash::Hash;

use bsnext_core::servers_supervisor::file_changed_handler::{FileChanged, FilesChanged};
use bsnext_dto::{StoppedWatchingDTO, WatchingDTO};
use bsnext_fs::watch_path_handler::RequestWatchPath;
use bsnext_fs::{
    BufferedChangeEvent, ChangeEvent, Debounce, FsEvent, FsEventKind, PathAddedEvent, PathEvent,
};
use bsnext_input::route::{DebounceDuration, DirRoute, FilterKind, RouteKind, Spec, SpecOpts};
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::{Input, InputCtx, InputError, PathDefinition, PathDefs, PathError};
use std::path::{Path, PathBuf};
use std::time::Duration;

use bsnext_fs::actor::FsWatcher;

use crate::input_fs::from_input_path;
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal::{AnyEvent, InternalEvents};
use bsnext_input::watch_opts::WatchOpts;

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
    pub ctx: InputCtx,
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
        let input_monitor = InputMonitor {
            addr: input_watcher_addr.clone(),
            ctx: msg.ctx.clone(),
        };
        self.input_monitors = Some(input_monitor);

        input_watcher_addr.do_send(RequestWatchPath {
            recipients: vec![ctx.address().recipient()],
            path: msg.path.to_path_buf(),
        });
    }
}

impl BsSystem {
    fn handle_buffered(&mut self, msg: &FsEvent, buf: &BufferedChangeEvent) -> Option<AnyEvent> {
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
        Some(AnyEvent::External(ExternalEventsDTO::FilesChanged(
            bsnext_dto::FilesChangedDTO { paths: as_strings },
        )))
    }
    fn handle_change(
        &mut self,
        msg: &FsEvent,
        inner: &ChangeEvent,
    ) -> Option<(AnyEvent, Option<Input>)> {
        match msg.ctx.id() {
            0 => {
                tracing::info!("InputFile file changed {:?}", inner);

                let ctx = self
                    .input_monitors
                    .as_ref()
                    .map(|x| x.ctx.clone())
                    .unwrap_or_default();

                let input = from_input_path(&inner.absolute_path, &ctx);

                let Ok(input) = input else {
                    let err = input.unwrap_err();
                    return Some((AnyEvent::Internal(InternalEvents::InputError(*err)), None));
                };

                if let Some(mon) = self.input_monitors.as_mut() {
                    let next = input
                        .servers
                        .iter()
                        .map(|s| s.identity.clone())
                        .collect::<Vec<_>>();
                    let ctx = InputCtx::new(&next, None);
                    tracing::debug!(?ctx);
                    if !next.is_empty() {
                        tracing::info!(
                            "updating stored server identities following a file change {:?}",
                            next
                        );
                        mon.ctx = ctx
                    }
                }

                Some((
                    AnyEvent::External(ExternalEventsDTO::InputFileChanged(
                        bsnext_dto::FileChangedDTO::from_path_buf(&inner.path),
                    )),
                    Some(input),
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
                Some((
                    AnyEvent::External(ExternalEventsDTO::FileChanged(
                        bsnext_dto::FileChangedDTO::from_path_buf(&inner.path),
                    )),
                    None,
                ))
            }
        }
    }
    fn handle_path_added(&mut self, path: &PathAddedEvent) -> Option<AnyEvent> {
        Some(AnyEvent::External(ExternalEventsDTO::Watching(
            WatchingDTO::from_path_buf(&path.path, path.debounce),
        )))
    }

    fn handle_path_removed(&mut self, path: &PathEvent) -> Option<AnyEvent> {
        Some(AnyEvent::External(ExternalEventsDTO::WatchingStopped(
            StoppedWatchingDTO::from_path_buf(&path.path),
        )))
    }

    fn handle_path_not_found(&mut self, pdo: &PathEvent) -> Option<AnyEvent> {
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
        Some(AnyEvent::Internal(InternalEvents::InputError(e)))
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
        todo!("implement resolve servers")
        // self.resolve_servers(msg.input);
    }
}

impl actix::Handler<FsEvent> for BsSystem {
    type Result = ();
    fn handle(&mut self, msg: FsEvent, ctx: &mut Self::Context) -> Self::Result {
        let next = match &msg.kind {
            FsEventKind::ChangeBuffered(buffer_change) => self.handle_buffered(&msg, buffer_change),
            FsEventKind::Change(inner) => {
                if let Some((evt, input)) = self.handle_change(&msg, inner) {
                    if let Some(input) = input {
                        // todo: add a test to ensure this is not removed
                        self.accept_watchables(&input);
                        ctx.address().do_send(ResolveServers { input });
                    }
                    Some(evt)
                } else {
                    None
                }
            }
            FsEventKind::PathAdded(path) => self.handle_path_added(path),
            FsEventKind::PathRemoved(path) => self.handle_path_removed(path),
            FsEventKind::PathNotFoundError(pdo) => self.handle_path_not_found(pdo),
        };
        if let Some(ext) = next {
            self.publish_any_event(ext)
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
                            route_path: r.path.as_str().to_owned(),
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
