use crate::task::TaskCommand;
use actix::ResponseFuture;
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal::{AnyEvent, ExitCode, InvocationId, TaskResult};
use bsnext_input::route::{PrefixOpt, ShRunOptItem};
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::ops::Deref;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

pub const DEFAULT_TERMINAL_OUTPUT_PREFIX: &str = "[run]";

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct ShCmd {
    sh: Cmd,
    name: Option<String>,
    output: ShCmdOutput,
    timeout: ShDuration,
}

impl Display for ShCmd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", (*self.sh).to_string_lossy())
    }
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
struct ShDuration(pub Duration);

impl ShDuration {
    pub fn duration(&self) -> &Duration {
        &self.0
    }
}

impl Default for ShDuration {
    fn default() -> Self {
        ShDuration(Duration::from_secs(60))
    }
}

#[derive(Debug, Default, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
enum ShCmdOutput {
    None,
    #[default]
    DefaultNamed,
    CustomNamed(String),
}

impl ShCmd {
    pub fn new(cmd: OsString) -> Self {
        Self {
            sh: Cmd(cmd),
            name: None,
            output: Default::default(),
            timeout: Default::default(),
        }
    }
    pub fn named(cmd: OsString, name: impl Into<String>) -> Self {
        Self {
            sh: Cmd(cmd),
            name: Some(name.into()),
            output: Default::default(),
            timeout: Default::default(),
        }
    }

    // pub fn from_opt(run_opt: &RunOptItem) -> {
    //
    // }

    pub fn named_prefix(&mut self, name: impl Into<String>) {
        self.output = ShCmdOutput::CustomNamed(name.into());
    }

    pub fn no_prefix(&mut self) {
        self.output = ShCmdOutput::None;
    }

    pub fn name(&self) -> Option<String> {
        self.name.as_ref().map(ToOwned::to_owned)
    }

    pub fn prefix(&self) -> Option<String> {
        match &self.output {
            ShCmdOutput::None => None,
            ShCmdOutput::DefaultNamed => match &self.name {
                None => Some(DEFAULT_TERMINAL_OUTPUT_PREFIX.to_string()),
                Some(sn_name) => Some(sn_name.clone()),
            },
            ShCmdOutput::CustomNamed(name) => Some(name.clone()),
        }
    }
}

impl From<ShRunOptItem> for ShCmd {
    fn from(value: ShRunOptItem) -> Self {
        let cmd: OsString = value.sh.into();
        let name = value.name;
        let mut sh = ShCmd::new(cmd);
        sh.name = name;
        match value.prefix {
            None => {}
            Some(PrefixOpt::Bool(true)) => {}
            Some(PrefixOpt::Bool(false)) => sh.no_prefix(),
            Some(PrefixOpt::Named(named)) => sh.named_prefix(named),
        }
        sh
    }
}

impl From<&ShRunOptItem> for ShCmd {
    fn from(value: &ShRunOptItem) -> Self {
        let cmd: OsString = value.sh.clone().into();
        let name = value.name.clone();
        let mut sh = ShCmd::new(cmd);
        sh.name = name;
        match &value.prefix {
            None => {}
            Some(PrefixOpt::Bool(true)) => {}
            Some(PrefixOpt::Bool(false)) => sh.no_prefix(),
            Some(PrefixOpt::Named(named)) => sh.named_prefix(named),
        }
        sh
    }
}

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
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
    type Result = ResponseFuture<TaskResult>;

    fn handle(&mut self, msg: TaskCommand, _ctx: &mut Self::Context) -> Self::Result {
        let cmd = self.sh.clone();
        let cmd = cmd.to_os_string();
        tracing::debug!("ShCmd: Will run... {:?}", cmd);
        let any_event_sender = msg.comms().any_event_sender.clone();
        let any_event_sender2 = msg.comms().any_event_sender.clone();
        let reason = match &msg {
            TaskCommand::Changes { changes, .. } => format!("{} files changed", changes.len()),
            TaskCommand::Exec { .. } => "command executed".to_string(),
        };
        let files = match &msg {
            TaskCommand::Changes { changes, .. } => changes
                .iter()
                .map(|x| format!("{}", x.display()))
                .collect::<Vec<_>>()
                .join(", "),
            TaskCommand::Exec { .. } => "NONE".to_string(),
        };

        let sh_prefix = Arc::new(self.prefix());
        let sh_prefix_2 = sh_prefix.clone();
        let max_duration = self.timeout.duration().to_owned();

        let fut = async move {
            let mut child = Command::new("sh")
                .kill_on_drop(true)
                .arg("-c")
                .arg(cmd)
                .env("TERM", "xterm-256color")
                .env("CLICOLOR_FORCE", "1")
                .env("FORCE_COLOR", "true")
                .env("CLICOLOR", "1")
                .env("COLORTERM", "truecolor")
                .env("BSLIVE_REASON", reason)
                .env("BSLIVE_FILES", files)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()
                .expect("command failed to spawn?");
            let pid = child.id();

            let stdout = child
                .stdout
                .take()
                .expect("child did not have a handle to stdout");

            let stderr = child
                .stderr
                .take()
                .expect("child did not have a handle to stderr");

            let mut stdout_reader = BufReader::new(stdout).lines();
            let mut stderr_reader = BufReader::new(stderr).lines();

            let h = tokio::spawn(async move {
                while let Ok(Some(line)) = stdout_reader.next_line().await {
                    match any_event_sender
                        .send(AnyEvent::External(ExternalEventsDTO::stdout_line(
                            line,
                            (*sh_prefix).clone(),
                        )))
                        .await
                    {
                        Ok(_) => tracing::trace!("did forward stdout line"),
                        Err(_) => tracing::error!("could not send stdout line"),
                    }
                }
            });
            let h2 = tokio::spawn(async move {
                while let Ok(Some(line)) = stderr_reader.next_line().await {
                    match any_event_sender2
                        .send(AnyEvent::External(ExternalEventsDTO::stderr_line(
                            line,
                            (*sh_prefix_2).clone(),
                        )))
                        .await
                    {
                        Ok(_) => tracing::trace!("did forward stdout line"),
                        Err(_) => tracing::error!("could not send stdout line"),
                    }
                }
            });

            let sleep = tokio::time::sleep(max_duration);

            tokio::pin!(sleep);
            let invocation_id = 0;

            let result: TaskResult = tokio::select! {
                _ = &mut sleep => {
                    tracing::info!("⌛️ operation timed out");
                    TaskResult::timeout(InvocationId(invocation_id))
                }
                out = child.wait() => {
                    tracing::info!("✅ operation ended");
                    match out {
                        Ok(exit) => match exit.code() {
                           Some(0) => TaskResult::ok(InvocationId(invocation_id)),
                           Some(code) => {
                                tracing::debug!("did exit with code {}", code);
                                TaskResult::err_code(InvocationId(invocation_id), ExitCode(code))
                            },
                           None => TaskResult::err_message(InvocationId(invocation_id), "unknown error!")
                        },
                        Err(err) => TaskResult::err_message(InvocationId(invocation_id), &err.to_string())
                    }
                }
            };
            if let Some(pid) = pid {
                let _ = kill_tree::tokio::kill_tree(pid).await;
                tracing::trace!("child tree killed");
            }

            match h.await {
                Ok(_) => tracing::trace!("did wait for stdout"),
                Err(e) => tracing::trace!("failed waiting for stdout {e}"),
            };
            match h2.await {
                Ok(_) => tracing::trace!("did wait for stderr"),
                Err(e) => tracing::trace!("failed waiting for stderr {e}"),
            };

            tracing::debug!("✅ Run (+cleanup) complete");

            result
        };
        Box::pin(fut)
    }
}
