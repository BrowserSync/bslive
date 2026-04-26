use actix::Actor;
use bsnext_fs::actor::FsWatcher;
use bsnext_fs::watch_path_handler::RequestWatchPath;
use bsnext_fs::{FsEvent, FsEventContext};
use bsnext_tracing::{
    init_tracing, LineNumberOption, LogHttp, LogLevel, OutputFormat, WriteOption,
};
use std::ffi::OsString;
use std::ops::Deref;
use std::path::PathBuf;
use std::time::Duration;

#[actix_rt::main]
async fn main() -> anyhow::Result<()> {
    let _g = init_tracing(
        Some(LogLevel::Trace),
        LogHttp::Off,
        OutputFormat::Normal,
        WriteOption::None,
        LineNumberOption::None,
    );
    let (File(file), Dir(dir)) = mock_path("mocks/01.txt");
    let ex = Example::from_str("echo 'hello world!'")?;
    let recip = ex.start();
    let fs = FsWatcher::new(&dir, FsEventContext::default(), recip.recipient());
    let addr = fs.start();
    addr.do_send(RequestWatchPath { path: file });
    tokio::time::sleep(Duration::from_secs(1000)).await;
    Ok(())
}

#[derive(Default)]
struct Example {
    running: bool,
    cmd: Cmd,
}

impl Example {
    pub fn from_str<A: AsRef<str>>(cmd: A) -> anyhow::Result<Self> {
        Ok(Self {
            running: false,
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
    type Result = ();

    fn handle(&mut self, msg: FsEvent, _ctx: &mut Self::Context) {
        tracing::info!(running = self.running, " ++ incoming");
        tracing::info!(cmd = self.cmd.0.to_str());
        tracing::info!(?msg.kind);
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
