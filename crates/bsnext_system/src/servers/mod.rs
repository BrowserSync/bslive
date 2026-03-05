use crate::system::BsSystem;
use actix::ResponseFuture;
use actix_rt::Arbiter;
use bsnext_core::server::handler_client_config::ClientConfigChange;
use bsnext_core::server::handler_routes_updated::RoutesUpdated;
use bsnext_core::servers_supervisor::actor::{ChildHandler, ChildStopped};
use bsnext_core::servers_supervisor::get_servers_handler::GetActiveServers;
use bsnext_core::servers_supervisor::input_changed_handler::InputChanged;
use bsnext_core::servers_supervisor::start_handler::ChildCreatedInsert;
use bsnext_dto::internal::{AnyEvent, ChildResult, InternalEvents, ServerError};
use bsnext_dto::GetActiveServersResponse;
use bsnext_input::Input;
use tracing::debug;

#[derive(actix::Message)]
#[rtype(result = "Result<(GetActiveServersResponse, Vec<ChildResult>), ServerError>")]
pub struct ResolveServers {
    pub(crate) input: Input,
}

impl actix::Handler<ResolveServers> for BsSystem {
    type Result = ResponseFuture<Result<(GetActiveServersResponse, Vec<ChildResult>), ServerError>>;

    #[tracing::instrument(skip_all, name = "Handler->ResolveServers->BsSystem")]
    fn handle(&mut self, msg: ResolveServers, _ctx: &mut Self::Context) -> Self::Result {
        let external_event_sender = self.sender().clone();
        let addr = self.servers().clone();

        let f = async move {
            debug!("will mark input as changed or new");
            let results = addr.send(InputChanged { input: msg.input }).await;

            let Ok(result_set) = results else {
                let e = results.unwrap_err();
                unreachable!("?1 {:?}", e);
            };

            debug!(
                "result_set from resolve servers {}",
                result_set.changes.len()
            );

            for (maybe_addr, x) in &result_set.changes {
                match x {
                    ChildResult::Stopped(id) => addr.do_send(ChildStopped {
                        identity: id.clone(),
                    }),
                    ChildResult::Created(c) if maybe_addr.is_some() => {
                        let child_handler = ChildHandler {
                            actor_address: maybe_addr.clone().expect("guarded above"),
                            identity: c.server_handler.identity.clone(),
                            socket_addr: c.server_handler.socket_addr,
                        };
                        addr.do_send(ChildCreatedInsert { child_handler })
                    }
                    ChildResult::Created(_c) => {
                        unreachable!("can't be created without")
                    }
                    ChildResult::Patched(p) if maybe_addr.is_some() => {
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

            let res = result_set
                .changes
                .into_iter()
                .map(|(_, child_result)| child_result)
                .collect::<Vec<_>>();

            match addr.send(GetActiveServers).await {
                Ok(resp) => {
                    Arbiter::current().spawn({
                        let evt = InternalEvents::ServersChanged {
                            server_resp: resp.clone(),
                            child_results: res.clone(),
                        };
                        debug!("will emit {:?}", evt);
                        async move {
                            match external_event_sender.send(AnyEvent::Internal(evt)).await {
                                Ok(_) => {}
                                Err(e) => debug!(?e),
                            };
                        }
                    });
                    Ok((resp, res))
                }
                Err(e) => Err(ServerError::Unknown(e.to_string())),
            }
        };

        Box::pin(f)
    }
}

#[derive(actix::Message)]
#[rtype(result = "Result<GetActiveServersResponse, ServerError>")]
pub struct ReadActiveServers;

impl actix::Handler<ReadActiveServers> for BsSystem {
    type Result = ResponseFuture<Result<GetActiveServersResponse, ServerError>>;

    fn handle(&mut self, _msg: ReadActiveServers, _ctx: &mut Self::Context) -> Self::Result {
        let cloned_address = self.servers().clone();

        Box::pin(async move {
            match cloned_address.send(GetActiveServers).await {
                Ok(resp) => Ok(resp),
                Err(e) => Err(ServerError::Unknown(e.to_string())),
            }
        })
    }
}
