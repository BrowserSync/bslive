use bsnext_core::shared_args::{FsOpts, InputOpts};
use bsnext_dto::internal::AnyEvent;
use bsnext_output::stdout::StdoutTarget;
use bsnext_output::OutputWriters;
use start_command::StartCommand;
use std::future::Future;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tracing::debug_span;

pub mod start_command;
pub mod start_kind;

pub fn stdout_channel(writer: OutputWriters) -> (Sender<AnyEvent>, impl Future<Output = ()>) {
    let (events_sender, mut events_receiver) = mpsc::channel::<AnyEvent>(1);
    let channel_future = async move {
        let stdout = &mut std::io::stdout();
        let stderr = &mut std::io::stderr();
        let mut sink = StdoutTarget::new(stdout, stderr);
        while let Some(evt) = events_receiver.recv().await {
            let span = debug_span!("External Event processor");
            let _g2 = span.enter();
            tracing::debug!(external_event = ?evt);
            let result = match evt {
                AnyEvent::Internal(int) => writer.write_evt(&int, &mut sink.output()),
                AnyEvent::External(ext) => writer.write_evt(&ext, &mut sink.output()),
            };
            match result {
                Ok(_) => {}
                Err(_) => tracing::error!("could not handle event"),
            }
            sink.flush();
        }
    };
    (events_sender, channel_future)
}

pub async fn with_stdout(
    cwd: PathBuf,
    fs_opts: &FsOpts,
    input_opts: &InputOpts,
    writer: OutputWriters,
    start: StartCommand,
) -> Result<(), anyhow::Error> {
    let (events_sender, channel_future) = stdout_channel(writer);
    let fs_opts_clone = fs_opts.clone();
    let input_opts_clone = input_opts.clone();
    let startup =
        start_command::start_cmd(cwd, fs_opts_clone, input_opts_clone, start, events_sender).await;
    match startup {
        Ok((handle, _sys_address)) => {
            let end = tokio::select! {
                r = handle => {
                    match r {
                        Ok(_) => Ok(()),
                        Err(er) => Err(anyhow::anyhow!("{:?}", er)),
                    }
                }
                h = actix_rt::spawn(channel_future) => {
                    match h {
                        Ok(_) => Ok(()),
                        Err(er) => Err(anyhow::anyhow!("{:?}", er))
                    }
                }
            };
            end
        }
        Err(err) => Err(anyhow::anyhow!("{:?}", err)),
    }
}
