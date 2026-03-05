use crate::api::BsSystemApi;
use crate::input_monitor::MonitorInput;
use crate::start::start_kind::StartKind;
use crate::system::{BsSystem, RunDryOk, RunOk, SetupOk};
use actix::{
    Actor, ActorContext, ActorFutureExt, AsyncContext, Handler, ResponseActFuture, WrapFuture,
};
use bsnext_dto::internal::InternalEvents::TaskSpecDisplay;
use bsnext_dto::internal::{AnyEvent, ChildResult, InternalEvents};
use bsnext_dto::{DidStart, StartupError};
use bsnext_input::startup::{RunMode, SystemStart, SystemStartArgs};
use bsnext_input::InputCtx;
use std::future::ready;
use std::path::PathBuf;
use tokio::sync::oneshot;
use tracing::debug;

pub async fn start_system(
    cwd: PathBuf,
    start_kind: StartKind,
    events_sender: tokio::sync::mpsc::Sender<AnyEvent>,
) -> Result<Option<BsSystemApi>, StartupError> {
    let (tx, rx) = oneshot::channel();
    let system = BsSystem::new(events_sender.clone(), cwd, tx);
    let sys_addr = system.start();

    tracing::debug!("{:?}", start_kind);

    let start = Start { kind: start_kind };

    match sys_addr.send(start).await {
        Ok(Ok(DidStart::Started(..))) => {
            tracing::debug!("DidStart::Started");
            let api = BsSystemApi::new(sys_addr, rx);
            Ok(Some(api))
        }
        Ok(Ok(DidStart::WillExit)) => {
            tracing::debug!("DidStart::WillExit");
            Ok(None)
        }
        Ok(Err(e)) => Err(e),
        Err(e) => {
            let message = e.to_string();
            Err(StartupError::Other(message))
        }
    }
}

#[derive(Debug, actix::Message)]
#[rtype(result = "Result<DidStart, StartupError>")]
pub struct Start {
    pub kind: StartKind,
}

impl Handler<Start> for BsSystem {
    type Result = ResponseActFuture<Self, Result<DidStart, StartupError>>;

    #[tracing::instrument(name = "BsSystem->Start", skip(self, msg, ctx))]
    fn handle(&mut self, msg: Start, ctx: &mut Self::Context) -> Self::Result {
        let addr = ctx.address();
        match msg.kind.input(&self.start_context) {
            Ok(SystemStartArgs::PathWithInput { path, input }) => {
                debug!("SystemStartArgs::PathWithInput");

                let ids = input.ids();
                let input_ctx = InputCtx::new(&ids, None, &self.start_context, Some(&path));
                let input_clone2 = input.clone();
                let jobs = crate::system::setup_jobs(addr.clone(), input.clone());

                Box::pin(jobs.into_actor(self).map(
                    move |res: Result<SetupOk, anyhow::Error>, actor, ctx| {
                        let SetupOk { servers, .. } = res.map_err(StartupError::Any)?;
                        debug!("✅ setup jobs completed");
                        ctx.notify(MonitorInput {
                            path: path.clone(),
                            cwd: actor.cwd.clone(),
                            input_ctx,
                        });
                        // todo: where to better sequence these side-effects?
                        actor.accept_watchables(&input_clone2, addr);
                        Ok(DidStart::Started(servers))
                    },
                ))
            }
            Ok(SystemStartArgs::InputOnly { input }) => {
                debug!("SystemStartArgs::InputOnly");

                let addr = ctx.address();
                let input_clone2 = input.clone();
                let jobs = crate::system::setup_jobs(addr.clone(), input.clone());

                Box::pin(jobs.into_actor(self).map(
                    move |res: Result<SetupOk, anyhow::Error>, actor, _ctx| {
                        let res = res?;
                        debug!("✅ setup jobs completed");
                        let errored = ChildResult::first_server_error(&res.child_results);
                        if let Some(server_error) = errored {
                            debug!("errored: {:?}", errored);
                            return Err(StartupError::ServerError((*server_error).to_owned()));
                        }
                        actor.accept_watchables(&input_clone2, addr);
                        Ok(DidStart::Started(res.servers))
                    },
                ))
            }
            Ok(SystemStartArgs::PathWithInvalidInput { path, input_error }) => {
                debug!("SystemStartArgs::PathWithInvalidInput");
                ctx.notify(MonitorInput {
                    path: path.clone(),
                    cwd: self.cwd.clone(),
                    input_ctx: InputCtx::default(),
                });
                self.publish_any_event(AnyEvent::Internal(InternalEvents::InputError(input_error)));
                let f = ready(Ok(DidStart::Started(Default::default()))).into_actor(self);
                Box::pin(f)
            }
            Ok(SystemStartArgs::RunOnly {
                input,
                named,
                run_mode: RunMode::Exec,
                top_level_run_mode,
            }) => {
                let addr = ctx.address();
                let jobs = crate::system::run_jobs(addr, input.clone(), named, top_level_run_mode);
                Box::pin(jobs.into_actor(self).map(
                    move |res: Result<RunOk, anyhow::Error>, _actor, _ctx| match res {
                        Ok(_) => Ok(DidStart::WillExit),
                        Err(err) => Err(StartupError::Any(err.into())),
                    },
                ))
            }
            Ok(SystemStartArgs::RunOnly {
                input,
                named,
                run_mode: RunMode::Dry,
                top_level_run_mode,
            }) => {
                let addr = ctx.address();
                let jobs =
                    crate::system::print_jobs(addr, input.clone(), named, top_level_run_mode);
                Box::pin(jobs.into_actor(self).map(
                    move |res: Result<RunDryOk, anyhow::Error>, actor, _ctx| match res {
                        Ok(RunDryOk { tree, spec: _ }) => {
                            actor.publish_any_event(AnyEvent::Internal(TaskSpecDisplay { tree }));
                            Ok(DidStart::WillExit)
                        }
                        Err(err) => Err(StartupError::Any(err.into())),
                    },
                ))
            }
            Err(e) => {
                let f = ready(Err(StartupError::InputError(*e))).into_actor(self);
                Box::pin(f)
            }
        }
    }
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct StopSystem;

impl Handler<StopSystem> for BsSystem {
    type Result = ();

    fn handle(&mut self, _msg: StopSystem, ctx: &mut Self::Context) -> Self::Result {
        tracing::trace!("handling StopSystem. Note: not graceful.");
        ctx.stop();
    }
}
