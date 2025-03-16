use actix::{Actor, ActorFutureExt, AsyncContext, ResponseActFuture, WrapFuture};
use actix_rt::Arbiter;
use bsnext_fs::actor::FsWatcher;
use bsnext_fs::filter::Filter;
use bsnext_fs::watch_path_handler::RequestWatchPath;
use bsnext_fs::{
    BufferedChangeEvent, Debounce, FsEvent, FsEventContext, FsEventKind, PathDescriptionOwned,
};
use std::fs;
use std::path::PathBuf;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;

#[actix_rt::main]
async fn main() {
    let str = bsnext_tracing::level(Some(bsnext_tracing::LogLevel::Trace));
    let with = format!("{str},watch_cmd=trace");
    bsnext_tracing::raw_tracing::init_tracing_subscriber(
        &with,
        bsnext_tracing::OutputFormat::Normal,
        bsnext_tracing::WriteOption::None,
    );
    let (File(file), Dir(dir)) = mock_path("mocks/01.txt");
    let mut fs = FsWatcher::new(&dir, FsEventContext::default());
    fs.with_debounce(Debounce::Trailing {
        duration: Duration::from_millis(300),
    });
    let addr = fs.start();
    let ex = Example::default();
    let recip = ex.start();
    addr.do_send(RequestWatchPath {
        path: file,
        recipients: vec![recip.recipient()],
    });
    tokio::time::sleep(Duration::from_secs(1000)).await;
}

#[derive(Default)]
struct Example {
    run_count: usize,
    running: bool,
}

impl Actor for Example {
    type Context = actix::Context<Self>;
}

impl actix::Handler<FsEvent> for Example {
    type Result = ResponseActFuture<Self, ()>;

    fn handle(&mut self, msg: FsEvent, ctx: &mut Self::Context) -> Self::Result {
        tracing::info!(running = self.running, " ++ incoming");
        let FsEvent { kind, .. } = msg else {
            todo!("Can this ever happen?")
        };
        let running = self.running;
        let valid_trigger = match kind {
            FsEventKind::Change(ch) => {
                tracing::debug!("    change {}", ch.path.display());
                true
            }
            FsEventKind::ChangeBuffered(BufferedChangeEvent { events }) => {
                tracing::debug!("    got {} events", events.len());
                for x in events {
                    tracing::debug!("      abs: {}", x.absolute.display());
                }
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
        self.running = true;
        let index = self.run_count + 1;
        tracing::debug!(index, "✍️ running = true");
        self.run_count = index;
        let fut = async move {
            tracing::debug!(index = index, "Will run...");
            let mut f1 = Command::new("bash")
                .arg("tree.sh")
                .kill_on_drop(true)
                .env("TERM", "xterm-256color") // Set terminal type
                .env("CLICOLOR_FORCE", "1") // Force colors in many Unix tools
                .env("CLICOLOR", "1") // Enable colors
                .env("COLORTERM", "truecolor") // Indicate full color support
                .stdout(Stdio::inherit())
                .spawn()
                .expect("ls command failed to start");
            let pid = f1.id();
            let sleep = tokio::time::sleep(Duration::from_secs(1));
            tokio::pin!(sleep);
            loop {
                tokio::select! {
                    _ = &mut sleep => {
                        println!("operation timed out");
                        break;
                    }
                    _ = f1.wait() => {
                        println!("operation completed");
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
                .map(|res, act, _ctx| {
                    tracing::debug!(index = act.run_count, "✍️ running = false");
                    act.running = false;
                    ()
                }),
        )
    }
}

#[derive(actix::Message, Debug)]
#[rtype(result = "()")]
struct Exec {
    cmd: Command,
    events: Vec<FsEvent>,
}

impl actix::Handler<Exec> for Example {
    type Result = ();

    fn handle(&mut self, msg: Exec, ctx: &mut Self::Context) -> Self::Result {
        tracing::debug!("will handle?");
        Arbiter::current().spawn(async move {
            tracing::debug!("Will run...");
            Command::new("sleep")
                .arg("5")
                .spawn()
                .expect("ls command failed to start")
                .wait()
                .await
                .expect("ls command failed to run");
            Command::new("echo")
                .arg("after")
                .spawn()
                .expect("echo command failed to start")
                .wait()
                .await
                .expect("echo command failed to run");
        });
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
