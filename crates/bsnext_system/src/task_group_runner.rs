use crate::runner::RunKind;
use crate::task::{AsActor, TaskCommand};
use crate::task_group::TaskGroup;
use actix::{ActorFutureExt, Handler, ResponseActFuture, Running, WrapFuture};
use bsnext_dto::internal::{ExpectedLen, InvocationId, TaskReport, TaskResult};
use futures_util::FutureExt;
use tokio::task::JoinSet;

pub struct TaskGroupRunner {
    done: bool,
    task_group: Option<TaskGroup>,
}

impl TaskGroupRunner {
    pub fn new(task_group: TaskGroup) -> Self {
        Self {
            task_group: Some(task_group),
            done: false,
        }
    }
}

impl actix::Actor for TaskGroupRunner {
    type Context = actix::Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        tracing::info!(actor.lifecycle = "started", "TaskGroupRunner2")
    }
    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        tracing::info!(" ⏰ stopping TaskGroupRunner2 {}", self.done);
        Running::Stop
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::info!(" x stopped TaskGroupRunner2")
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
        tracing::debug!("  └── {:?} group run_kind", group.run_kind());
        let expected_len = group.len();

        let future = async move {
            let mut done: Vec<(usize, TaskReport)> = vec![];
            match group.run_kind() {
                RunKind::Sequence => {
                    for (index, group_item) in group.tasks().into_iter().enumerate() {
                        let id = group_item.id();
                        let x = Box::new(group_item).into_actor2();
                        match x.send(msg.clone()).await {
                            Ok(result) => {
                                let is_ok = result.is_ok();
                                done.push((index, result.to_report(id)));
                                if is_ok {
                                    tracing::debug!(
                                        "index {index} completed, will move to next text in seq"
                                    );
                                } else {
                                    tracing::debug!("stopping after index {index} id: {id} ran, because it did not complete successfully.");
                                    break;
                                }
                            }
                            Err(e) => tracing::error!("{e}"),
                        }
                    }
                }
                RunKind::Overlapping => {
                    let futures = group
                        .tasks()
                        .into_iter()
                        .enumerate()
                        .map(|(index, as_actor)| {
                            let id = as_actor.id();
                            let a = Box::new(as_actor).into_actor2();
                            a.send(msg.clone())
                                .map(move |task_result| (index, id, task_result))
                        });
                    let mut set = JoinSet::from_iter(futures);
                    while let Some(res) = set.join_next().await {
                        match res {
                            Ok((index, id, Ok(result))) => {
                                done.push((index, result.to_report(id)));
                            }
                            Ok((index, _, Err(mailbox_error))) => {
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
        Box::pin(future.into_actor(self).map(move |res, actor, _ctx| {
            actor.done = true;

            tracing::info!("actual len: {}", res.len());
            tracing::info!("expected len: {}", expected_len);

            let all_good = res.iter().all(|(_index, report)| report.result().is_ok());
            let all_ran = res.len() == expected_len;
            let reports: Vec<_> = res.into_iter().map(|(_, report)| report).collect();

            let failed_only = reports
                .iter()
                .filter(|x| !x.result().is_ok())
                .map(Clone::clone)
                .collect::<Vec<_>>();

            tracing::debug!(result.all_good = all_good, result.all_ran = all_ran);

            match (all_ran, all_good) {
                (true, true) => TaskResult::ok_tasks(InvocationId(0), reports),
                (true, false) => TaskResult::err_tasks(InvocationId(0), failed_only, reports),
                (false, _) => TaskResult::err_partial_tasks(
                    InvocationId(0),
                    reports,
                    ExpectedLen(expected_len),
                ),
            }
        }))
    }
}
