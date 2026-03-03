use crate::capabilities::Capabilities;
use actix::Addr;
use bsnext_core::servers_supervisor::actor::ServersSupervisor;

#[derive(Debug, Clone)]
pub struct Comms {
    pub servers_addr: Option<Addr<ServersSupervisor>>,
    pub capabilities: Addr<Capabilities>,
}
