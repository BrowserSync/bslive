use crate::capabilities::Capabilities;
use actix::Addr;

#[derive(Debug, Clone)]
pub struct Comms {
    pub capabilities: Addr<Capabilities>,
}
