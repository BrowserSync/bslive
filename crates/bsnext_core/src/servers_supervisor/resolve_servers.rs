use crate::server::handler_client_config::ClientConfigChange;
use crate::server::handler_routes_updated::RoutesUpdated;
use crate::servers_supervisor::actor::{ChildHandler, ChildStopped, ServersSupervisor};
use crate::servers_supervisor::get_servers_handler::GetActiveServers;
use crate::servers_supervisor::input_changed_handler::{InputChanged, InputChangedResponse};
use actix::ActorFutureExt;
use actix::{Addr, AsyncContext, ResponseActFuture, ResponseFuture, WrapFuture};
use bsnext_dto::any_event::AnyEvent;
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::server_events::{ChildResult, ServerError};
use bsnext_dto::status_events::{
    active_server_status_hash, ServerStatusReader, ServerStatusWriter, ServersStatus, StateValue,
};
use bsnext_dto::{GetActiveServersResponse, ServerChangesetDTO};
use bsnext_input::Input;
use std::convert::Infallible;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::time::Duration;
use tracing::debug;

#[derive(actix::Message)]
#[rtype(result = "Result<(), ServerError>")]
pub struct ResolveServers;

impl ResolveServers {
    pub fn new() -> Self {
        ResolveServers
    }
}

impl actix::Handler<ResolveServers> for ServersSupervisor {
    type Result = ResponseActFuture<Self, Result<(), ServerError>>;

    #[tracing::instrument(skip_all, name = "Handler->ResolveServers->ServersSupervisor")]
    fn handle(&mut self, msg: ResolveServers, ctx: &mut Self::Context) -> Self::Result {
        let addr = ctx.address();
        let sender = self.sender.clone();
        let local_status = self.status;
        let state_writer = self.state_writer.clone();
        let state_reader = self.state_reader.clone();

        let f = async move {
            tracing::info!("will ask for server input...");
            let Ok((sys_server_status, input)) = state_reader.send(ServerStatusReader).await else {
                unreachable!("can it fail?")
            };

            if let (ServersStatus::Ready { .. }, ServersStatus::Ready { .. }) =
                (sys_server_status, local_status)
            {
                tracing::info!("doing nothing, sys + local state match");
                tracing::info!(?sys_server_status);
                tracing::info!(?local_status);

                return Ok(None);
            }

            tracing::info!("There's something to reconcile...");

            let expected_id = match sys_server_status {
                ServersStatus::Idle => todo!("how can we get here?"),
                ServersStatus::Reconciling { desired, .. } => desired,
                ServersStatus::Ready { .. } => todo!("how...?"),
            };

            let results = addr.send(InputChanged { input }).await;

            let Ok(result_set) = results else {
                let e = results.unwrap_err();
                unreachable!("?1 {:?}", e);
            };

            Ok::<Option<(StateValue, InputChangedResponse)>, ServerError>(Some((
                expected_id,
                result_set,
            )))
        };

        Box::pin(f.into_actor(self).map(move |res, act, ctx| {
            match res {
                Err(e) => {
                    tracing::info!(e = ?e, "got error?");
                    return Ok(());
                }
                Ok(None) => {
                    tracing::info!("got None, returning with nothing to do...?");
                    return Ok(());
                }
                Ok(Some((desired, result_set))) => {
                    tracing::info!("got result_set");
                    debug!(
                        "result_set from resolve servers {}",
                        result_set.changes.len()
                    );

                    for (maybe_addr, x) in &result_set.changes {
                        match x {
                            ChildResult::Stopped(id) => {
                                act.handlers.remove(id);
                            }
                            ChildResult::Created(c) if maybe_addr.is_some() => {
                                let child_handler = ChildHandler {
                                    actor_address: maybe_addr.clone().expect("guarded above"),
                                    identity: c.server_handler.identity.clone(),
                                    socket_addr: c.server_handler.socket_addr,
                                    content_hash: c.server_handler.content_hash,
                                };
                                act.handlers
                                    .insert(child_handler.identity.clone(), child_handler.clone());
                            }
                            ChildResult::Created(_c) => {
                                unreachable!("can't be created without")
                            }
                            ChildResult::Patched(p) if maybe_addr.is_some() => {
                                // todo: must mark new content_id...
                                dbg!("must mark local content_hash as the resolved one");
                                tracing::info!(
                                    content_hash = p.server_handler.content_hash,
                                    next_content_hash = p.next_content_hash,
                                    "next--"
                                );
                                if let Some(t) = act.handlers.get_mut(&p.server_handler.identity) {
                                    t.content_hash = p.next_content_hash;
                                }
                                if let Some(child_actor) = maybe_addr {
                                    child_actor.do_send(ClientConfigChange {
                                        change_set: p.client_config_change_set.clone(),
                                    });
                                    child_actor.do_send(RoutesUpdated {
                                        change_set: p.route_change_set.clone(),
                                    })
                                } else {
                                    tracing::error!("missing actor addr where it was needed")
                                }
                            }
                            ChildResult::Patched(p) => {
                                debug!("ChildResult::Patched {:?}", p);
                            }
                            ChildResult::PatchErr(e) => {
                                debug!("ChildResult::PatchErr {:?}", e);
                            }
                            ChildResult::CreateErr(e) => {
                                debug!("ChildResult::CreateErr {:?}", e);
                            }
                        }
                    }

                    let res: Vec<ChildResult> = result_set
                        .changes
                        .into_iter()
                        .map(|(_, child_result)| child_result)
                        .collect();

                    let resp = act.active_servers();
                    let observed = StateValue::new(active_server_status_hash(&resp.servers));
                    let len = resp.servers.len();
                    if observed == desired {
                        act.status = ServersStatus::Ready { desired, observed };
                        state_writer.do_send(ServerStatusWriter {
                            server_status: act.status,
                        });
                    } else {
                        act.status = ServersStatus::Reconciling {
                            desired,
                            observed: Some(observed),
                        };
                    }
                    tracing::info!(observed = ?act.status, len, ?desired, "observed");

                    ctx.notify_later(ResolveServers, Duration::from_secs(2));

                    // if the current state matches the desired state, we're done for now, shedule a new check
                    // if observed.eq(desired) {
                    //
                    // }

                    // let dto = ServerChangesetDTO::from_changes(&resp, &res);
                    // dbg!(dto);

                    // let _ = sender
                    //     .send(AnyEvent::External(ExternalEventsDTO::ServerChangeset(dto)))
                    //     .await;
                }
            }
            Ok(())
        }))
    }
}

async fn get_servers(
    addr: Addr<ServersSupervisor>,
    results: Vec<ChildResult>,
) -> anyhow::Result<(GetActiveServersResponse, Vec<ChildResult>)> {
    let resp = addr.send(GetActiveServers).await?;
    Ok((resp, results))
}
