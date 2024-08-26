use crate::server::actor::ServerActor;
use crate::server::router::make_router;
use crate::server::state::ServerState;
use crate::servers_supervisor::get_servers_handler::{GetServersMessage, IncomingEvents};
use actix::{Recipient, ResponseFuture};
use actix_rt::Arbiter;
use axum::Router;
use bsnext_dto::internal::ServerError;
use bsnext_input::server_config::ServerIdentity;
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use tokio::sync::{oneshot, RwLock};

#[derive(actix::Message)]
#[rtype(result = "Result<SocketAddr, ServerError>")]
pub struct Listen {
    pub parent: Recipient<GetServersMessage>,
    pub evt_receiver: Recipient<IncomingEvents>,
}

impl actix::Handler<Listen> for ServerActor {
    type Result = ResponseFuture<Result<SocketAddr, ServerError>>;

    fn handle(&mut self, msg: Listen, _ctx: &mut Self::Context) -> Self::Result {
        let identity = self.config.identity.clone();
        tracing::trace!("actor started for {:?}", identity);
        let (send_complete, handle, client_sender) = self.install_signals();
        let (oneshot_send, oneshot_rec) = oneshot::channel();
        let h1 = handle.clone();
        let h2 = handle.clone();

        let app_state = Arc::new(ServerState {
            // parent: ,
            routes: Arc::new(RwLock::new(self.config.routes.clone())),
            router: Arc::new(RwLock::new(Router::new())),
            id: self.config.identity.as_id(),
            parent: Some(msg.parent.clone()),
            evt_receiver: Some(msg.evt_receiver.clone()),
            client_sender: Arc::new(client_sender),
        });

        self.app_state = Some(app_state.clone());

        let server = async move {
            let router = make_router(&app_state);

            let maybe_socket_addr: Result<SocketAddr, _> = match identity {
                ServerIdentity::Both {
                    ref bind_address, ..
                } => bind_address.parse(),
                ServerIdentity::Address { ref bind_address } => bind_address.parse(),
                ServerIdentity::Named { .. } => {
                    format!("127.0.0.1:{}", get_available_port().expect("port?")).parse()
                }
            };

            let Ok(socket_addr) = maybe_socket_addr else {
                tracing::error!(
                    "{:?} [❌ NOT started] could not parse bind_address",
                    identity
                );

                match oneshot_send.send(Err(ServerError::InvalidAddress {
                    addr_parse_error: maybe_socket_addr.unwrap_err().to_string(),
                })) {
                    Ok(_) => {}
                    Err(_) => tracing::error!("oneshot send failed"),
                }
                return;
            };

            tracing::trace!("trying to listen on {:?}", socket_addr);

            let server = axum_server::bind(socket_addr)
                .handle(h1)
                .serve(router.into_make_service_with_connect_info::<SocketAddr>());

            let result: Result<_, ServerError> = match server.await {
                Ok(_) => {
                    tracing::debug!("{:?} [started] Server all done", identity);
                    if send_complete.send(()).is_err() {
                        tracing::error!("{:?} [started] could not send complete message", identity);
                    }
                    Ok(())
                }
                Err(e) => match e.kind() {
                    ErrorKind::AddrInUse => {
                        tracing::error!("{:?} [not-started] [AddrInUse] {}", identity, e);
                        Err(ServerError::AddrInUse { socket_addr })
                    }
                    _ => {
                        tracing::error!("{:?} [not-started] UNKNOWN {}", identity, e);
                        Err(ServerError::Unknown(format!("{}", e)))
                    }
                },
            };
            if !oneshot_send.is_closed() {
                let _r = oneshot_send.send(result);
            }
        };

        Arbiter::current().spawn(server);

        Box::pin(async move {
            tokio::select! {
                listening = h2.listening() => {
                    match listening {
                        Some(socket_addr) => {
                            tracing::debug!("{} listening...", socket_addr);
                            Ok(socket_addr)
                        }
                        None => {
                            Err(ServerError::Unknown("unknown".to_string()))
                        }
                    }
                }
                msg = oneshot_rec => {
                    match msg {
                        Ok(v) => {
                            match v {
                                Ok(_) => {
                                    tracing::info!("All good from one_shot?");
                                    Err(ServerError::Closed)
                                }
                                Err(e) => {
                                    Err(e)
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("-->{e}");
                            Err(ServerError::Unknown(format!("{:?}", e)))
                        }
                    }
                }
            }
        })
    }
}

pub fn get_available_port() -> Option<u16> {
    TcpListener::bind("127.0.0.1:0")
        .and_then(|listener| listener.local_addr())
        .map(|socket_addr| socket_addr.port())
        .ok()
}

#[derive(actix::Message)]
#[rtype(result = "()")]
pub struct Listening {
    addr: SocketAddr,
}

impl actix::Handler<Listening> for ServerActor {
    type Result = ();

    fn handle(&mut self, msg: Listening, _ctx: &mut Self::Context) -> Self::Result {
        self.addr = Some(msg.addr);
    }
}
