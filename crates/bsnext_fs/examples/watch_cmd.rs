use actix::{Actor, ActorFutureExt, ResponseActFuture, WrapFuture};
use bsnext_fs::actor::FsWatcher;
use bsnext_fs::watch_path_handler::RequestWatchPath;
use bsnext_fs::{FsEvent, FsEventContext, FsEventKind};
use std::ffi::OsString;
use std::ops::Deref;
use std::path::PathBuf;
use std::time::Duration;
use tokio::process::Command;

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    let str = bsnext_tracing::level(Some(bsnext_tracing::LogLevel::Trace));
    let with = format!("{str},watch_cmd=trace");
    bsnext_tracing::raw_tracing::init_tracing_subscriber(
        &with,
        bsnext_tracing::OutputFormat::Normal,
        bsnext_tracing::WriteOption::None,
        bsnext_tracing::LineNumberOption::None,
    );
    let (File(file), Dir(dir)) = mock_path("mocks/01.txt");
    let ex = Example::from_str("echo 'hello world!' && printenv")?;
    let recip = ex.start();
    let fs = FsWatcher::new(&dir, FsEventContext::default(), recip.recipient());
    let addr = fs.start();
    addr.do_send(RequestWatchPath { path: file });
    tokio::time::sleep(Duration::from_secs(1000)).await;
    Ok(())
}

#[derive(Default)]
struct Example {
    run_count: usize,
    running: bool,
    cmd: Cmd,
}

impl Example {
    pub fn from_str<A: AsRef<str>>(cmd: A) -> anyhow::Result<Self> {
        Ok(Self {
            running: false,
            run_count: 0,
            cmd: Cmd(OsString::try_from(cmd.as_ref())?),
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

impl Default for Cmd {
    fn default() -> Self {
        Self("echo 'default command - did you forget to give a command?'".into())
    }
}

impl Actor for Example {
    type Context = actix::Context<Self>;
}

impl actix::Handler<FsEvent> for Example {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: FsEvent, _ctx: &mut Self::Context) -> Self::Result {
        tracing::info!(running = self.running, " ++ incoming");
        tracing::info!(cmd = self.cmd.0.to_str());
        let FsEvent { kind, .. } = msg;
        let running = self.running;
        let valid_trigger = match kind {
            FsEventKind::Change(ch) => {
                tracing::debug!("    change {:?}", ch.relative);
                true
            }
            FsEventKind::PathAdded(path) => {
                tracing::debug!("    PathAdded {} ", path.path.display());
                false
            }
            FsEventKind::PathRemoved(_) => {
                tracing::debug!("    PathRemoved");
                false
            }
            FsEventKind::PathNotFoundError(_) => {
                tracing::debug!("    PathNotFoundError");
                false
            }
        };
        let will_run = valid_trigger && !running;
        tracing::debug!(valid_trigger, running, "will possibly run?");
        if !will_run {
            tracing::debug!(valid_trigger, "will not run for this");
            return Box::pin(
                async {
                    ();
                }
                .into_actor(self),
            );
        };
        let index = self.run_count + 1;
        let cmd = self.cmd.to_os_string();
        self.running = true;
        tracing::debug!(index, "✍️ running = true");
        self.run_count = index;
        let fut = async move {
            tracing::debug!(index = index, "Will run...");
            let mut f1 = Command::new("sh")
                .kill_on_drop(true)
                .arg("-c")
                .arg(cmd)
                .env("TERM", "xterm-256color") // Set terminal type
                .env("CLICOLOR_FORCE", "1") // Force colors in many Unix tools
                .env("CLICOLOR", "1") // Enable colors
                .env("COLORTERM", "truecolor") // Indicate full color support
                .spawn()
                .expect("ls command failed to start");
            let pid = f1.id();
            let sleep = tokio::time::sleep(Duration::from_secs(10));
            tokio::pin!(sleep);
            loop {
                tokio::select! {
                    _ = &mut sleep => {
                        println!("⌛️ operation timed out");
                        break;
                    }
                    _ = f1.wait() => {
                        println!("✅ operation completed");
                        break;
                    }
                }
            }
            if let Some(pid) = pid {
                let _ = kill_tree::tokio::kill_tree(pid).await;
                println!("child tree killed");
            }
            tracing::debug!("✅ Run complete");
        };

        Box::pin(
            fut.into_actor(self) // converts future to ActorFuture
                .map(|_res, act, _ctx| {
                    tracing::debug!(index = act.run_count, "✍️ running = false");
                    act.running = false;
                    ()
                }),
        )
    }
}

struct Dir(PathBuf);
struct File(PathBuf);

fn mock_path(a: &str) -> (File, Dir) {
    let buf = PathBuf::from(file!()).canonicalize().unwrap();
    let dir = buf.parent().unwrap();
    let mock1 = dir.join(a);
    (File(mock1), Dir(dir.to_path_buf()))
}
