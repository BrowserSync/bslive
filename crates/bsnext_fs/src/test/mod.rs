use crate::actor::FsWatcher;
use crate::watch_path_handler::RequestWatchPath;
use crate::{Debounce, FsEvent, FsEventContext, FsEventKind};
use actix::{Actor, Addr};
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;

use crate::filter::Filter;
use tokio::time::sleep;

struct A {
    events: Vec<FsEvent>,
}

impl Actor for A {
    type Context = actix::Context<Self>;
}
#[derive(actix::Message)]
#[rtype(result = "Vec<FsEvent>")]
struct GetEvents;

impl actix::Handler<GetEvents> for A {
    type Result = Vec<FsEvent>;

    fn handle(&mut self, _msg: GetEvents, _ctx: &mut Self::Context) -> Self::Result {
        self.events.clone()
    }
}
impl actix::Handler<FsEvent> for A {
    type Result = ();

    fn handle(&mut self, msg: FsEvent, _ctx: &mut Self::Context) -> Self::Result {
        self.events.push(msg);
    }
}

fn create_file(base: &Path, name: &str) -> PathBuf {
    let file_path = base.join(name);
    let mut file = File::create(&file_path).expect("create file");
    file.write_all(name.as_bytes())
        .expect(format!("write {name}").as_str());
    file_path
}

struct TestCase {
    addr: Addr<FsWatcher>,
    recip_addr: Addr<A>,
    dir: PathBuf,
    #[allow(dead_code)]
    tmp_dir: TempDir,
}

impl TestCase {
    pub fn new(debounce: Debounce, filter: Option<Filter>) -> Self {
        let tmp_dir = tempfile::tempdir().unwrap();
        let mut fs = FsWatcher::new(
            tmp_dir.path(),
            FsEventContext {
                id: 0,
                origin_id: 0,
            },
        );
        fs.with_debounce(debounce);
        if let Some(filter) = filter {
            fs.with_filter(filter);
        }
        let addr = fs.start();
        let a = A { events: vec![] };
        let recip_addr = a.start();
        Self {
            recip_addr: recip_addr.clone(),
            addr: addr.clone(),
            dir: tmp_dir.path().to_path_buf(),
            tmp_dir,
        }
    }

    async fn watch(&self) {
        let r = RequestWatchPath {
            recipients: vec![self.recip_addr.clone().recipient()],
            path: self.dir.to_path_buf(),
        };

        let _ = self.addr.send(r).await;
    }

    async fn write_file(&self, p: &str) {
        create_file(self.dir.as_path(), p);
    }

    async fn get_events_after(&self, d: Duration) -> Vec<FsEvent> {
        sleep(d).await;
        let events = self.recip_addr.send(GetEvents).await.unwrap();
        events
    }
    async fn change_events_after(&self, d: Duration) -> (Vec<FsEvent>, Vec<FsEvent>, usize) {
        sleep(d).await;
        let events = self.recip_addr.send(GetEvents).await.unwrap();
        let len = events.len();
        let (change_events, other_events) = events
            .into_iter()
            .partition(|e| matches!(e.kind, FsEventKind::Change(..)));
        (change_events, other_events, len)
    }
    async fn buffered_change_after(&self, d: Duration) -> (Vec<FsEvent>, Vec<FsEvent>, usize) {
        sleep(d).await;
        let events = self.recip_addr.send(GetEvents).await.unwrap();
        let len = events.len();
        let (change_events, other_events) = events
            .into_iter()
            .partition(|e| matches!(e.kind, FsEventKind::ChangeBuffered(..)));
        (change_events, other_events, len)
    }

    fn file_names(events: Vec<FsEvent>) -> Vec<String> {
        events
            .into_iter()
            .flat_map(|evt| match evt.kind {
                FsEventKind::Change(_) => vec![],
                FsEventKind::ChangeBuffered(buf) => buf
                    .events
                    .iter()
                    .map(|p| p.absolute.to_path_buf())
                    .collect(),
                FsEventKind::PathAdded(_) => vec![],
                FsEventKind::PathRemoved(_) => vec![],
                FsEventKind::PathNotFoundError(_) => vec![],
            })
            .map(|pb| pb.file_name().unwrap().to_string_lossy().to_string())
            .collect()
    }
}

#[actix_rt::test]
async fn test_single_file() -> Result<(), Box<dyn std::error::Error>> {
    let tc = TestCase::new(Debounce::trailing_ms(10), None);
    tc.watch().await;
    tc.write_file("test_file.txt").await;
    let events = tc.get_events_after(Duration::from_millis(500)).await;
    assert_eq!(events.len(), 2);
    assert_eq!(
        matches!(events.get(0).unwrap().kind, FsEventKind::PathAdded(..)),
        true
    );
    assert_eq!(
        matches!(events.get(1).unwrap().kind, FsEventKind::Change(..)),
        true
    );
    Ok(())
}

#[actix_rt::test]
async fn test_trailing_drops() -> Result<(), Box<dyn std::error::Error>> {
    let tc = TestCase::new(Debounce::trailing_ms(10), None);
    tc.watch().await;
    tc.write_file("test_file.txt").await;
    tc.write_file("test_file.css").await;
    let (change, other, _total_count) = tc.change_events_after(Duration::from_millis(500)).await;
    assert_eq!(change.len(), 1, "Should be a single change event");
    assert_eq!(other.len(), 1, "Should be 2 in total");
    Ok(())
}

#[actix_rt::test]
async fn test_buffer() -> Result<(), Box<dyn std::error::Error>> {
    let tc = TestCase::new(Debounce::buffered_ms(10), None);
    tc.watch().await;
    tc.write_file("test_file.txt").await;
    tc.write_file("test_file.css").await;
    let (change, other, total_count) = tc.buffered_change_after(Duration::from_millis(500)).await;
    assert_eq!(change.len(), 1, "Should be 1 change event (buffered)");
    assert_eq!(other.len(), 1, "Should be 1 other event (path added)");
    assert_eq!(total_count, 2, "Should be 2 in total");

    let names = TestCase::file_names(change);
    assert!(names.contains(&"test_file.txt".to_string()));
    assert!(names.contains(&"test_file.css".to_string()));
    Ok(())
}

#[actix_rt::test]
async fn test_buffer_filter() -> Result<(), Box<dyn std::error::Error>> {
    let tc = TestCase::new(
        Debounce::buffered_ms(10),
        Some(Filter::Extension {
            ext: "css".to_string(),
        }),
    );
    tc.watch().await;
    tc.write_file("test_file.txt").await;
    tc.write_file("test_file.css").await;
    let (change, other, total_count) = tc.buffered_change_after(Duration::from_millis(500)).await;
    assert_eq!(change.len(), 1, "Should be 1 change event (buffered)");
    assert_eq!(other.len(), 1, "Should be 1 other event (path added)");
    assert_eq!(total_count, 2, "Should be 2 in total");

    let names = TestCase::file_names(change);
    assert_eq!(names.len(), 1);
    assert!(names.contains(&"test_file.css".to_string()));
    Ok(())
}
