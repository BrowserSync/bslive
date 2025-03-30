use crate::task::TaskCommand;
use actix::ResponseFuture;
use std::ffi::OsString;
use std::ops::Deref;
use std::time::Duration;
use tokio::process::Command;

pub struct ShCmd {
    sh: Cmd,
}

impl ShCmd {
    pub fn new(cmd: OsString) -> Self {
        Self { sh: Cmd(cmd) }
    }
    #[allow(dead_code)]
    pub fn from_str<A: AsRef<str>>(cmd: A) -> anyhow::Result<Self> {
        Ok(Self {
            sh: Cmd(OsString::try_from(cmd.as_ref())?),
        })
    }
}

#[derive(Debug, Clone)]
struct Cmd(pub OsString);

impl Deref for Cmd {
    type Target = OsString;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl actix::Actor for ShCmd {
    type Context = actix::Context<Self>;
}

impl actix::Handler<TaskCommand> for ShCmd {
    type Result = ResponseFuture<()>;

    fn handle(&mut self, msg: TaskCommand, _ctx: &mut Self::Context) -> Self::Result {
        let cmd = self.sh.clone();
        let cmd = cmd.to_os_string();
        tracing::debug!("ShCmd: Will run... {:?}", cmd);
        let reason = match &msg {
            TaskCommand::Changes { changes, .. } => format!("{} files changed", changes.len()),
        };
        let files = match &msg {
            TaskCommand::Changes { changes, .. } => changes
                .iter()
                .map(|x| format!("{}", x.display()))
                .collect::<Vec<_>>()
                .join(", "),
        };
        let fut = async move {
            let mut f1 = Command::new("sh")
                .kill_on_drop(true)
                .arg("-c")
                .arg(cmd)
                .env("TERM", "xterm-256color")
                .env("CLICOLOR_FORCE", "1")
                .env("CLICOLOR", "1")
                .env("COLORTERM", "truecolor")
                .env("BSLIVE_REASON", reason)
                .env("BSLIVE_FILES", files)
                .spawn()
                .expect("command failed to spawn?");
            let pid = f1.id();
            // todo: where to encode things like this?
            let sleep = tokio::time::sleep(Duration::from_secs(10));
            tokio::pin!(sleep);
            tokio::select! {
                _ = &mut sleep => {
                    tracing::info!("⌛️ operation timed out");
                }
                _ = f1.wait() => {
                    tracing::info!("✅ operation completed");
                }
            }
            if let Some(pid) = pid {
                let _ = kill_tree::tokio::kill_tree(pid).await;
                tracing::trace!("child tree killed");
            }
            tracing::debug!("✅ Run complete");
        };
        Box::pin(fut)
    }
}
