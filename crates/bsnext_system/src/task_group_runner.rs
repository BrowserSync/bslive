use crate::runner::RunKind;
use crate::task::{
    AsActor, ExpectedLen, InvocationId, TaskCommand, TaskGroup, TaskResult, TaskStatus,
};
use actix::{ActorFutureExt, Handler, ResponseActFuture, Running, WrapFuture};
use futures_util::FutureExt;
use tokio::task::JoinSet;

pub struct TaskGroupRunner {
    done: bool,
    task_group: Option<TaskGroup>,
}

impl TaskGroupRunner {
    pub fn new(p0: TaskGroup) -> Self {
        Self {
            task_group: Some(p0),
            done: false,
        }
    }
}

impl actix::Actor for TaskGroupRunner {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::info!(actor.lifecycle = "started", "TaskGroupRunner2")
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::info!(" x stopped TaskGroupRunner2")
    }

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        tracing::info!(" ⏰ stopping TaskGroupRunner2 {}", self.done);
        Running::Stop
    }
}

impl Handler<TaskCommand> for TaskGroupRunner {
    type Result = ResponseActFuture<Self, TaskResult>;

    fn handle(&mut self, msg: TaskCommand, _ctx: &mut Self::Context) -> Self::Result {
        tracing::debug!(done = self.done, "TaskGroupRunner::TaskCommand");
        let Some(group) = self.task_group.take() else {
            todo!("how to handle a concurrent request here?");
        };
        tracing::debug!("  └── {} tasks in group", group.len());
        tracing::debug!("  └── {:?} group run_kind", group.run_kind);
        let expected_len = group.len();

        let future = async move {
            let actors = group
                .tasks
                .into_iter()
                .map(|x| x.into_actor2())
                .collect::<Vec<_>>();
            let mut done: Vec<(usize, TaskResult)> = vec![];
            match group.run_kind {
                RunKind::Sequence => {
                    for (index, x) in actors.iter().enumerate() {
                        match x.send(msg.clone()).await {
                            Ok(result) => {
                                let is_ok = result.is_ok();
                                done.push((index, result));
                                if is_ok {
                                    tracing::debug!(
                                        "index {index} completed, will move to next text in seq"
                                    );
                                } else {
                                    tracing::debug!("stopping after index {index} ran, because it did not complete successfully.");
                                    break;
                                }
                            }
                            Err(e) => tracing::error!("{e}"),
                        }
                    }
                }
                RunKind::Overlapping => {
                    let futs = actors
                        .into_iter()
                        .enumerate()
                        .map(|(index, a)| a.send(msg.clone()).map(move |r| (index, r)));
                    let mut set = JoinSet::from_iter(futs);
                    while let Some(res) = set.join_next().await {
                        match res {
                            Ok((index, Ok(result))) => done.push((index, result)),
                            Ok((index, Err(mailbox_error))) => {
                                tracing::error!(
                                    "could not send index: {}, {:?}",
                                    index,
                                    mailbox_error
                                );
                            }
                            Err(_) => tracing::error!("could not push index"),
                        }
                    }
                }
            };
            done
        };
        Box::pin(future.into_actor(self).map(move |res, actor, ctx| {
            actor.done = true;

            tracing::info!("actual len: {}", res.len());
            tracing::info!("expected len: {}", expected_len);

            let all_good = res.iter().all(|(_index, result)| result.is_ok());
            let all_ran = res.len() == expected_len;
            let results = res.into_iter().map(|(i, x)| x).collect();

            tracing::debug!("all_good={all_good}, all_ran={all_ran}");

            match (all_ran, all_good) {
                (true, true) => TaskResult::ok_tasks(InvocationId(0), results),
                (true, false) => TaskResult::err_tasks(InvocationId(0), results),
                (false, _) => TaskResult::err_partial_tasks(
                    InvocationId(0),
                    results,
                    ExpectedLen(expected_len),
                ),
            }
        }))
    }
}
