use crate::dto::ClientEvent;
use crate::server::actor::ServerActor;

use std::path::{Path, PathBuf};

#[derive(actix::Message, Clone, Debug)]
#[rtype(result = "()")]
pub enum Change {
    Fs {
        path: PathBuf,
        change_kind: ChangeKind,
    },
    FsMany(Vec<Change>),
}

#[typeshare::typeshare]
#[derive(Clone, Debug, serde::Serialize)]
pub enum ChangeKind {
    Changed,
    Added,
    Removed,
}

impl Change {
    pub fn fs<A: AsRef<Path>>(a: A) -> Self {
        Self::Fs {
            path: a.as_ref().to_path_buf(),
            change_kind: ChangeKind::Changed,
        }
    }
    pub fn fs_many<A: AsRef<Path>>(a: &[A]) -> Self {
        Self::FsMany(
            a.iter()
                .map(|p| p.as_ref().to_owned())
                .map(|p| Self::Fs {
                    path: p,
                    change_kind: ChangeKind::Changed,
                })
                .collect(),
        )
    }
    pub fn fs_added<A: AsRef<Path>>(a: A) -> Self {
        Self::Fs {
            path: a.as_ref().to_path_buf(),
            change_kind: ChangeKind::Added,
        }
    }
    pub fn fs_removed<A: AsRef<Path>>(a: A) -> Self {
        Self::Fs {
            path: a.as_ref().to_path_buf(),
            change_kind: ChangeKind::Removed,
        }
    }
}

impl actix::Handler<Change> for ServerActor {
    type Result = ();

    fn handle(&mut self, msg: Change, _ctx: &mut Self::Context) -> Self::Result {
        if let Some(sender) = self.signals.as_ref().and_then(|s| s.client_sender.as_ref()) {
            // todo: what messages are the clients expecting?
            tracing::info!("forwarding `Change` event to connected web socket clients");
            match sender.send(ClientEvent::Change((&msg).into())) {
                Ok(_) => {
                    tracing::trace!("change event sent to clients");
                }
                Err(_) => tracing::error!("not sent to client_sender"),
            };
        } else {
            tracing::debug!("signals not ready, should they be?");
        }
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::dto::ChangeDTO;
    #[test]
    fn test_serialize() -> anyhow::Result<()> {
        let fs: ChangeDTO = (&Change::fs("./a.js")).into();
        let json = serde_json::to_string(&fs).unwrap();
        print!("{json}");
        Ok(())
    }
}
