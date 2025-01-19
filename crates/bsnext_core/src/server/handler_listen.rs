use crate::handler_stack::RouteMap;
use crate::runtime_ctx::RuntimeCtx;
use crate::server::actor::ServerActor;
use crate::server::router::make_router;
use crate::server::state::ServerState;
use crate::servers_supervisor::get_servers_handler::{GetActiveServers, IncomingEvents};
use actix::{Recipient, ResponseFuture};
use actix_rt::Arbiter;
use bsnext_dto::internal::ServerError;
use bsnext_input::server_config::ServerIdentity;
use std::io::ErrorKind;
use std::net::{SocketAddr, TcpListener};
use std::sync::Arc;
use tokio::sync::{oneshot, RwLock};

#[derive(actix::Message)]
#[rtype(result = "Result<SocketAddr, ServerError>")]
pub struct Listen {
    pub parent: Recipient<GetActiveServers>,
    pub evt_receiver: Recipient<IncomingEvents>,
    pub runtime_ctx: RuntimeCtx,
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

        let router =
            RouteMap::new_from_routes(&self.config.combined_routes()).into_router(&msg.runtime_ctx);

        let app_state = Arc::new(ServerState {
            // parent: ,
            routes: Arc::new(RwLock::new(self.config.combined_routes())),
            raw_router: Arc::new(RwLock::new(router)),
            runtime_ctx: msg.runtime_ctx,
            client_config: Arc::new(RwLock::new(self.config.clients.clone())),
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
                ServerIdentity::Port { port } | ServerIdentity::PortNamed { port, .. } => {
                    let address = SocketAddr::new(
                        std::net::IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1)),
                        port,
                    );
                    Ok(address)
                }
            };

            let Ok(socket_addr) = maybe_socket_addr else {
                tracing::debug!(
                    "{:?} [❌ NOT started] could not parse bind_address",
                    identity
                );

                match oneshot_send.send(Err(ServerError::InvalidAddress {
                    addr_parse_error: maybe_socket_addr.unwrap_err().to_string(),
                })) {
                    Ok(_) => {}
                    Err(_) => tracing::debug!("❌ oneshot send failed"),
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
                        tracing::debug!(
                            "❌ {:?} [started] could not send complete message",
                            identity
                        );
                    }
                    Ok(())
                }
                Err(e) => match e.kind() {
                    ErrorKind::AddrInUse => {
                        tracing::debug!("❌ {:?} [not-started] [AddrInUse] {}", identity, e);
                        Err(ServerError::AddrInUse { socket_addr })
                    }
                    _ => {
                        tracing::debug!("❌ {:?} [not-started] UNKNOWN {}", identity, e);
                        Err(ServerError::Unknown(format!("{}", e)))
                    }
                },
            };
            if !oneshot_send.is_closed() {
                let _r = oneshot_send.send(result);
            } else {
                tracing::debug!("a channel was closed? {:?}", result);
            }
        };

        Arbiter::current().spawn(server);

        Box::pin(async move {
            let listen_r = h2.listening().await;

            if let Some(socket_addr) = listen_r {
                return Ok(socket_addr);
            };

            let once = oneshot_rec.await;
            match once {
                Ok(Ok(_)) => Err(ServerError::Closed),
                Ok(Err(server_error)) => Err(server_error),
                Err(other) => {
                    tracing::error!("-->{other}");
                    Err(ServerError::Unknown(format!("{:?}", other)))
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
