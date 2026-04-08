use crate::capabilities::output_channel::RequestOutputChannel;
use crate::capabilities::{Capabilities, TaggedEvent};
use crate::tasks::into_recipient::IntoRecipient;
use actix::{Actor, Addr, Recipient, ResponseFuture};
use bsnext_dto::external_events::ExternalEventsDTO;
use bsnext_dto::internal::AnyEvent;
use bsnext_input::route::{PrefixOpt, ShRunOptItem};
use bsnext_task::invocation::Invocation;
use bsnext_task::invocation_result::InvocationResult;
use bsnext_task::task_report::ExitCode;
use bsnext_task::task_trigger::TaskTriggerSource;
use bsnext_task::NodePath;
use std::ffi::OsString;
use std::fmt::{Display, Formatter};
use std::hash::Hash;
use std::ops::Deref;
use std::process::Stdio;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use tracing::Instrument;

pub const DEFAULT_TERMINAL_OUTPUT_PREFIX: &str = "[run]";

#[derive(Debug, PartialEq, PartialOrd, Ord, Eq, Hash, Clone)]
pub struct ShCmd {
    sh: Cmd,
    name: Option<String>,
    output: ShCmdOutput,
    timeout: ShDuration,
    id: Option<NodePath>,
}

struct ShCmdWithLogging {
    cmd: ShCmd,
    request_sender: Recipient<RequestOutputChannel>,
}

impl IntoRecipient for ShCmd {
    fn into_recipient(self, addr: &Addr<Capabilities>) -> Recipient<Invocation> {
        let with_logging = ShCmdWithLogging {
            cmd: self,
            request_sender: addr.clone().recipient(),
        };
        let actor_address = with_logging.start();
        actor_address.recipient()
    }
}

impl Display for ShCmd {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "ShCmd {}", (*self.sh).to_string_lossy())
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
            id: None,
        }
    }

    pub fn named_prefix(&mut self, name: impl Into<String>) {
        self.output = ShCmdOutput::CustomNamed(name.into());
    }

    pub fn no_prefix(&mut self) {
        self.output = ShCmdOutput::None;
    }

    pub fn name(&self) -> Option<String> {
        self.name.as_ref().map(ToOwned::to_owned)
    }

    pub fn prefix(&self, node_path: &NodePath) -> Option<String> {
        match &self.output {
            ShCmdOutput::None => None,
            ShCmdOutput::DefaultNamed => match &self.name {
                None => Some(format!("{node_path}")),
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

impl actix::Actor for ShCmdWithLogging {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(actor.name = "ShCmd", actor.lifecyle = "started");
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(actor.name = "ShCmd", actor.lifecyle = "stopped");
    }
}

impl actix::Handler<Invocation> for ShCmdWithLogging {
    type Result = ResponseFuture<InvocationResult>;

    #[tracing::instrument(skip_all, name = "sh_cmd::invocation")]
    fn handle(&mut self, invocation: Invocation, _ctx: &mut Self::Context) -> Self::Result {
        let path = invocation.path().to_owned();
        self.cmd.id = Some(path.clone());
        let cmd = self.cmd.sh.clone();
        let trigger = invocation.trigger().to_owned();
        let node_path = invocation.path().to_owned();
        let cmd = cmd.to_os_string();

        tracing::info!("Will run... {:?}", cmd);
        // let any_event_sender = comms.any_event_sender.clone();
        // let any_event_sender2 = comms.any_event_sender.clone();
        let reason = match &trigger.source() {
            TaskTriggerSource::FsChanges(trigger) => {
                format!("{} files changed", trigger.changes().len())
            }
            TaskTriggerSource::Exec { .. } => "command executed".to_string(),
        };

        let files = match &trigger.source() {
            TaskTriggerSource::FsChanges(trigger) => trigger
                .changes()
                .iter()
                .map(|x| format!("{}", x.display()))
                .collect::<Vec<_>>()
                .join(", "),
            TaskTriggerSource::Exec(..) => "NONE".to_string(),
        };

        let sh_prefix = Arc::new(self.cmd.prefix(&path));
        let sh_prefix_2 = sh_prefix.clone();
        let max_duration = self.cmd.timeout.duration().to_owned();
        let addr = self.request_sender.clone();

        let fut = sh_cmd(
            addr,
            node_path,
            cmd,
            reason,
            files,
            sh_prefix,
            sh_prefix_2,
            max_duration,
        )
        .in_current_span();

        Box::pin(fut)
    }
}

#[tracing::instrument(skip_all)]
async fn sh_cmd(
    addr: Recipient<RequestOutputChannel>,
    node_path: NodePath,
    cmd: OsString,
    reason: String,
    files: String,
    sh_prefix: Arc<Option<String>>,
    sh_prefix_2: Arc<Option<String>>,
    max_duration: Duration,
) -> InvocationResult {
    let Ok(Ok(output)) = addr.send(RequestOutputChannel).await else {
        todo!("can this actually fail?");
    };
    let sender = output.sender.clone();
    let sender2 = output.sender.clone();

    let mut command = if cfg!(target_os = "windows") {
        let mut c = Command::new("cmd");
        c.arg("/C");
        c
    } else {
        let mut c = Command::new("sh");
        c.arg("-c");
        c
    };

    let mut child = command
        .kill_on_drop(true)
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
    tracing::debug!(?pid);

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
        tracing::debug!(?pid, "reading stdout");
        while let Ok(Some(line)) = stdout_reader.next_line().await {
            match sender
                .send(TaggedEvent::new(AnyEvent::External(
                    ExternalEventsDTO::stdout_line(line, (*sh_prefix).clone()),
                )))
                .await
            {
                Ok(_) => tracing::trace!("did forward stdout line"),
                Err(_) => tracing::error!("could not send stdout line"),
            }
        }
    });

    let h2 = tokio::spawn(async move {
        tracing::debug!(?pid, "reading stderr");
        while let Ok(Some(line)) = stderr_reader.next_line().await {
            match sender2
                .send(TaggedEvent::new(AnyEvent::External(
                    ExternalEventsDTO::stderr_line(line, (*sh_prefix_2).clone()),
                )))
                .await
            {
                Ok(_) => tracing::trace!("did forward stderr line"),
                Err(_) => tracing::error!("could not send stderr line"),
            }
        }
    });

    let deadline = tokio::time::sleep(max_duration);

    tokio::pin!(deadline);

    let result: InvocationResult = tokio::select! {
        _ = &mut deadline => {
            tracing::info!("⌛️ operation timed out");
            InvocationResult::timeout(node_path)
        }
        out = child.wait() => {
            tracing::info!("child waited");
            match out {
                Ok(exit) => match exit.code() {
                   Some(0) => InvocationResult::ok(node_path),
                   Some(code) => {
                        tracing::debug!("did exit with code {}", code);
                        InvocationResult::err_code(node_path, ExitCode(code))
                    },
                   None => InvocationResult::err_message(node_path, "unknown error!")
                },
                Err(err) => InvocationResult::err_message(node_path, &err.to_string())
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

    tracing::info!("✅ complete");

    result
}
