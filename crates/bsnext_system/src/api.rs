use crate::servers::ReadActiveServers;
use crate::start::start_system::StopSystem;
use crate::BsSystem;
use actix::Addr;
use bsnext_dto::internal::ServerError;
use bsnext_dto::ActiveServer;
use bsnext_fs::{FsEvent, FsEventGrouping};
use tokio::sync::oneshot;

#[derive(Debug)]
pub struct BsSystemApi {
    sys_address: Addr<BsSystem>,
    handle: oneshot::Receiver<()>,
}

impl BsSystemApi {
    pub fn new(sys_address: Addr<BsSystem>, handle: oneshot::Receiver<()>) -> Self {
        Self {
            sys_address,
            handle,
        }
    }
}

impl BsSystemApi {
    ///
    /// Stop the system entirely. Note: this consumes self
    /// and you cannot interact with the system
    ///
    pub async fn stop(&self) -> anyhow::Result<()> {
        self.sys_address
            .send(StopSystem)
            .await
            .map_err(|e| anyhow::anyhow!("could not stop: {:?}", e))
    }
    ///
    /// Use this to keep the server open
    ///
    pub async fn handle(self) -> anyhow::Result<()> {
        self.handle
            .await
            .map_err(|e| anyhow::anyhow!("could not wait: {:?}", e))
    }

    pub fn fs_event(&self, evt: FsEvent) {
        self.sys_address.do_send(FsEventGrouping::Singular(evt))
    }

    pub async fn active_servers(&self) -> Result<Vec<ActiveServer>, ServerError> {
        match self.sys_address.send(ReadActiveServers).await {
            Ok(Ok(resp)) => Ok(resp.servers),
            _ => {
                tracing::error!("Could not send ReadActiveServers to sys_address");
                Err(ServerError::Unknown(
                    "Could not send ReadActiveServers to sys_address".to_string(),
                ))
            }
        }
    }
}
