use crate::start::start_kind::StartKind;
use crate::{BsSystem, Start};
use actix::{Actor, Addr};
use bsnext_core::shared_args::{FsOpts, InputOpts};
use bsnext_dto::internal::{AnyEvent, StartupEvent};
use bsnext_input::startup::{DidStart, StartupError};
use bsnext_output::OutputWriterTrait;
use std::path::PathBuf;
use tokio::sync::oneshot;

#[derive(Debug, Clone, clap::Parser)]
pub struct StartCommand {
    /// Should permissive cors headers be added to responses?
    #[arg(long)]
    pub cors: bool,

    /// Only works with `--example` - specify a port instead of a random one
    #[arg(short, long)]
    pub port: Option<u16>,

    /// Paths to serve + possibly watch, incompatible with `-i` option
    pub trailing: Vec<String>,
}

pub async fn start_cmd(
    cwd: PathBuf,
    fs_opts: FsOpts,
    input_opts: InputOpts,
    start_command: StartCommand,
    events_sender: tokio::sync::mpsc::Sender<AnyEvent>,
) -> Result<(oneshot::Receiver<()>, Addr<BsSystem>), impl OutputWriterTrait> {
    let (tx, rx) = oneshot::channel();
    let system = BsSystem::new();
    let sys_addr = system.start();
    let start_kind = StartKind::from_args(&fs_opts, &input_opts, &start_command);

    tracing::debug!(?start_kind);

    let start = Start {
        kind: start_kind,
        cwd: Some(cwd),
        ack: tx,
        events_sender,
    };

    match sys_addr.send(start).await {
        Ok(Ok(DidStart::Started)) => Ok((rx, sys_addr)),
        Ok(Err(e)) => Err(StartupEvent::FailedStartup(e)),
        Err(e) => {
            let message = e.to_string();
            Err(StartupEvent::FailedStartup(StartupError::Other(message)))
        }
    }
}
