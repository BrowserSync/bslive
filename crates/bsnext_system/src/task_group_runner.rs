use crate::as_actor::AsActor;
use crate::task_group::TaskGroup;
use crate::task_list::RunKind;
use crate::tasks::sh_cmd::OneTask;
use actix::{ActorFutureExt, Handler, MailboxError, ResponseActFuture, Running, WrapFuture};
use bsnext_dto::internal::{ExpectedLen, InvocationId, TaskReport, TaskResult};
use futures_util::FutureExt;
use std::sync::Arc;
use tokio::select;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tracing::{debug, trace, Instrument, Span};

/// Represents a task group runner responsible for managing and executing a set of tasks.
///
/// The `TaskGroupRunner` struct is used to handle task groups and their execution lifecycle.
/// It maintains the state of execution and the associated `TaskGroup`.
///
/// # Fields
///
/// * `done` - A boolean flag indicating whether the task group has finished execution.
/// * `task_group` - An `Option` containing the `TaskGroup` instance that this runner manages.
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
        tracing::trace!(actor.name = "TaskGroupRunner", actor.lifecycle = "started")
    }
    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        tracing::trace!(actor.name = "TaskGroupRunner", " ⏰ stopping {}", self.done);
        Running::Stop
    }
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        tracing::trace!(actor.name = "TaskGroupRunner", " x stopped")
    }
}

impl Handler<OneTask> for TaskGroupRunner {
    type Result = ResponseActFuture<Self, TaskResult>;

    #[tracing::instrument(skip_all, name = "OneTask", fields(id = one.sqid()))]
    fn handle(&mut self, one: OneTask, _ctx: &mut Self::Context) -> Self::Result {
        let OneTask(id, trigger) = one;
        let span = Span::current();
        let gg = Arc::new(span.clone());
        let ggg = gg.clone();

        debug!(done = self.done);

        let Some(group) = self.task_group.take() else {
            todo!("how to handle a concurrent request here?");
        };

        tracing::info!(group.len = group.len(), group.kind = ?group.run_kind());

        let expected_len = group.len();
        let future = async move {
            let mut done: Vec<(usize, TaskReport)> = vec![];
            let _e = ggg.enter();
            match group.run_kind() {
                RunKind::Sequence { opts } => {
                    let exit_on_failure = opts.exit_on_failure;
                    for (index, group_item) in group.tasks().into_iter().enumerate() {
                        let id = group_item.id();
                        let boxed_actor = Box::new(group_item).into_task_recipient();
                        let one_task = OneTask(id, trigger.clone());

                        match boxed_actor.send(one_task).await {
                            Ok(result) => {
                                let is_ok = result.is_ok();
                                done.push((index, result.to_report(id)));
                                if is_ok {
                                    debug!(
                                        "index {index} completed, will move to next text in seq"
                                    );
                                } else if exit_on_failure {
                                    debug!("❌ stopping after index {index} id: {id} ran, because it did not complete successfully.");
                                    break;
                                } else {
                                    debug!("continuing after failure, because of config");
                                }
                            }
                            Err(e) => tracing::error!("{e}"),
                        }
                    }
                }
                RunKind::Overlapping { opts } => {
                    #[derive(Clone, Copy)]
                    enum CancelOthers {
                        True,
                        False,
                    };
                    let fail_early = if opts.exit_on_failure {
                        CancelOthers::True
                    } else {
                        CancelOthers::False
                    };
                    let token = CancellationToken::new();
                    let sem = Arc::new(Semaphore::new(opts.max_concurrent_items as usize));
                    let mut jhs = Vec::new();
                    for (index, as_actor) in group.tasks().into_iter().enumerate() {
                        let parent_token = token.clone();
                        let child_token = parent_token.child_token();
                        let id = as_actor.id();
                        let actor = Box::new(as_actor).into_task_recipient();
                        let jh = tokio::spawn({
                            let semaphore = sem.clone();
                            let one_task = OneTask(id, trigger.clone());
                            async move {
                                if child_token.is_cancelled() {
                                    let v = TaskResult::cancelled();
                                    return (index, id, Ok::<_, _>(v));
                                };
                                let _permit = semaphore.acquire().await.unwrap();
                                let task_run = actor
                                    .send(one_task)
                                    .instrument(Span::current())
                                    .map(move |task_result| (index, id, task_result));
                                let output = select! {
                                    result = task_run => {
                                        match (&result, fail_early) {
                                            ((_, _, Ok(res)), CancelOthers::True) if !res.is_ok() => {
                                                debug!("cancelling group because CancelOthers::True");
                                                parent_token.cancel();
                                            }
                                            ((_, _, Ok(res)), CancelOthers::False) if !res.is_ok() => {
                                                debug!("NOT cancelling group because CancelOthers::False");
                                            }
                                            _ => {
                                                todo!("how does this occur?")
                                            }
                                        }
                                        result
                                    }
                                    _ = child_token.cancelled() => {
                                        debug!("child_token was cancelled");
                                        let v = TaskResult::cancelled();
                                        (index, id, Ok::<_, _>(v))
                                    }
                                };
                                drop(_permit);
                                output
                            }
                        });
                        jhs.push(jh);
                    }

                    for jh in jhs {
                        match jh.await {
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
            let _e = gg.enter();
            debug!("actual len: {}", res.len());
            debug!("expected len: {}", expected_len);

            let all_good = res.iter().all(|(_index, report)| report.result().is_ok());
            let all_ran = res.len() == expected_len;
            let reports: Vec<_> = res.into_iter().map(|(_, report)| report).collect();

            let failed_only = reports
                .iter()
                .filter(|x| !x.result().is_ok())
                .map(Clone::clone)
                .collect::<Vec<_>>();

            debug!(result.all_good = all_good, result.all_ran = all_ran);

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
