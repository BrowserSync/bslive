use actix::Actor;
use bsnext_fs::actor::FsWatcher;
use bsnext_fs::filter::Filter;
use bsnext_fs::watch_path_handler::RequestWatchPath;
use bsnext_fs::{BufferedChangeEvent, Debounce, FsEvent, FsEventContext, FsEventKind};
use std::fs;
use std::path::PathBuf;
use std::time::Duration;

#[actix_rt::main]
async fn main() {
    let (File(file), Dir(dir)) = mock_path("mocks/01.txt");
    let mut fs = FsWatcher::new(&dir, FsEventContext::default());
    fs.with_debounce(Debounce::Buffered {
        duration: Duration::from_millis(300),
    });
    let addr = fs.start();
    let ex = Example;
    let recip = ex.start();
    addr.send(RequestWatchPath {
        path: dir,
        recipients: vec![recip.recipient()],
    })
    .await;
    tokio::time::sleep(Duration::from_secs(1000)).await;
}

struct Example;

impl Actor for Example {
    type Context = actix::Context<Self>;
}

impl actix::Handler<FsEvent> for Example {
    type Result = ();

    fn handle(&mut self, msg: FsEvent, _ctx: &mut Self::Context) -> Self::Result {
        let FsEvent { kind, ctx } = msg else { todo!() };
        match kind {
            FsEventKind::Change(_) => {}
            FsEventKind::ChangeBuffered(BufferedChangeEvent { events }) => {
                println!("got {} events", events.len());
            }
            FsEventKind::PathAdded(_) => {}
            FsEventKind::PathRemoved(_) => {}
            FsEventKind::PathNotFoundError(_) => {}
        }
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
