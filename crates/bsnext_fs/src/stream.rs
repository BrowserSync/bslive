use core::pin::Pin;
use core::task::{Context, Poll};
use std::fmt::Debug;
use std::time::Duration;

use futures::{Future, Stream};
use pin_project_lite::pin_project;

use tokio::time::{Instant, Sleep};

use tracing::trace;

pin_project! {
    #[must_use = "streams do nothing unless polled"]
    pub struct Debounce<St: Stream> {
        #[pin]
        dropped_count: usize,
        #[pin]
        value: St,
        #[pin]
        delay: Sleep,
        #[pin]
        debounce_time: Duration,
        #[pin]
        last_state: Option<St::Item>,
        #[pin]
        child_ended: bool
    }
}

pub trait StreamOpsExt: Stream {
    fn debounce(self, debounce_time: Duration) -> Debounce<Self>
    where
        Self: Sized + Unpin,
    {
        Debounce::new(self, debounce_time)
    }
}

impl<T: ?Sized> StreamOpsExt for T where T: Stream {}

impl<St> Debounce<St>
where
    St: Stream + Unpin,
{
    #[allow(dead_code)]
    fn new(stream: St, debounce_time: Duration) -> Debounce<St> {
        Debounce {
            value: stream,
            dropped_count: 0,
            delay: tokio::time::sleep(debounce_time),
            debounce_time,
            last_state: None,
            child_ended: false,
        }
    }
}

impl<St, Item> Stream for Debounce<St>
where
    St: Stream<Item = Item>,
    Item: Clone + Unpin + Debug + 'static,
{
    type Item = St::Item;
    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut me = self.project();
        trace!("+ polled!");

        match me.value.poll_next(cx) {
            Poll::Ready(Some(v)) => {
                trace!("recorded a child value");
                *me.last_state = Some(v);
                *me.dropped_count += 1;

                trace!("resetting the deadline to be `debounce_time` from `now`");
                let dur = *me.debounce_time;
                me.delay.as_mut().reset(Instant::now() + dur);

                trace!("wake to ensure we're polled again");
                cx.waker().wake_by_ref();
                return Poll::Pending;
            }
            Poll::Ready(None) => {
                trace!("child ended, nothing more to do?");
                *me.child_ended = true;
            }
            Poll::Pending => {
                trace!("child was pending, nothing to do");
            }
        }

        match me.delay.poll(cx) {
            Poll::Ready(_) => {
                trace!("timer elapsed");
                match (*me.last_state).clone() {
                    Some(v) => {
                        trace!("buffered {} events", me.dropped_count);
                        *me.last_state = None;
                        *me.dropped_count = 0;
                        trace!("sending value");
                        Poll::Ready(Some(v))
                    }
                    None => {
                        if *me.child_ended {
                            Poll::Ready(None)
                        } else {
                            Poll::Pending
                        }
                    }
                }
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use tokio::sync::mpsc;
    use tokio::time::sleep;
    use tokio::time::Instant;
    use tokio_stream::wrappers::ReceiverStream;
    use tokio_stream::StreamExt;
    #[tokio::test]
    async fn test_stream() {
        let (tx, rx) = mpsc::channel::<&str>(10);
        let handle = tokio::spawn(async move {
            let events = ["A", "B", "C", "D", "E", "F"];

            // 6 events all happening together
            for evt in events {
                tx.send(evt).await.unwrap();
            }

            // a gap in events, just under the debounce duration
            sleep(Duration::from_millis(50)).await;

            tx.send("G").await.unwrap();
            tx.send("H").await.unwrap();
            tx.send("I").await.unwrap();
            tx.send("J").await.unwrap();

            // drop the sender to complete the stream
            drop(tx);
        });

        let start_time = Instant::now();
        let stream = Box::pin(
            ReceiverStream::new(rx)
                .debounce(Duration::from_millis(100))
                .collect::<Vec<_>>(),
        );

        let results = stream.await;
        let end_time = Instant::now();
        let total_duration = end_time
            .checked_duration_since(start_time)
            .expect("checked");

        println!("{:?}", results);
        println!("duration: {:?}", total_duration);

        assert!(handle.await.is_ok());
        assert_eq!(vec!["J"], results);
    }
}
