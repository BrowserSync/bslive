use crate::RunKind;
use crate::as_actor::AsActor;
use crate::invocation::Invocation;
use crate::task_scope::TaskScope;
use actix::{ActorFutureExt, Handler, ResponseActFuture, Running, WrapFuture};
use bsnext_dto::internal::{ExpectedLen, InvocationId, TaskReport, TaskResult};
use futures_util::FutureExt;
use std::sync::Arc;
use tokio::select;
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tracing::{Instrument, Span, debug};

/// Represents a task group runner responsible for managing and executing a set of tasks.
///
/// The `TaskGroupRunner` struct is used to handle task groups and their execution lifecycle.
/// It maintains the state of execution and the associated `TaskGroup`.
///
/// # Fields
///
/// * `done` - A boolean flag indicating whether the task group has finished execution.
/// * `task_group` - An `Option` containing the `TaskGroup` instance that this runner manages.
pub struct TaskScopeRunner {
    done: bool,
    task_scope: Option<TaskScope>,
}

impl TaskScopeRunner {
    pub fn new(task_group: TaskScope) -> Self {
        Self {
            task_scope: Some(task_group),
            done: false,
        }
    }
}

impl actix::Actor for TaskScopeRunner {
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

impl Handler<Invocation> for TaskScopeRunner {
    type Result = ResponseActFuture<Self, TaskResult>;

    #[tracing::instrument(skip_all, name = "Invocation", fields(id = invocation.sqid()))]
    fn handle(&mut self, invocation: Invocation, _ctx: &mut Self::Context) -> Self::Result {
        let sqid = invocation.sqid();
        let Invocation(_id, trigger) = invocation;
        let span = Span::current();
        let gg = Arc::new(span.clone());
        let ggg = gg.clone();

        debug!(done = self.done);

        let Some(group) = self.task_scope.take() else {
            todo!("how to handle a concurrent request here?");
        };

        let exit_on_failure = group.exit_on_failure();

        tracing::info!(
            group.sqid = sqid,
            group.len = group.len(),
            group.kind = ?group.run_kind(),
            group.exit_on_failure = exit_on_failure,
        );

        let expected_len = group.len();
        let future = async move {
            let mut done: Vec<(usize, TaskReport)> = vec![];
            let _e = ggg.enter();
            match group.run_kind() {
                RunKind::Sequence { opts: _ } => {
                    for (index, group_item) in group.tasks().into_iter().enumerate() {
                        let id = group_item.id();
                        let boxed_actor = Box::new(group_item).into_task_recipient();
                        let one_task = Invocation(id, trigger.clone());
                        let sqid = one_task.sqid();

                        match boxed_actor.send(one_task).await {
                            Ok(result) => {
                                let is_ok = result.is_ok();
                                done.push((index, result.to_report(id)));
                                if is_ok {
                                    debug!(
                                        "index {index} completed, will move to next text in seq"
                                    );
                                } else if exit_on_failure {
                                    debug!(
                                        "❌ stopping after index {index} sqid: {sqid} ran, because it did not complete successfully."
                                    );
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
                    }
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
                            let one_task = Invocation(id, trigger.clone());
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
                                            ((_, _, Ok(res)), _) => {
                                                debug!("doing nothing {:?}", res);
                                            }
                                            ((_, _, Err(e)), _a) => {
                                                todo!("how does this occur? {:?}", e)
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

            debug!(
                result.sqid = sqid,
                result.all_good = all_good,
                result.all_ran = all_ran,
                result.exit_on_failure = exit_on_failure
            );

            match (all_ran, all_good, exit_on_failure) {
                (true, true, _) => TaskResult::ok_tasks(InvocationId(0), reports),
                (true, false, true) => TaskResult::err_tasks(InvocationId(0), failed_only, reports),
                (true, false, false) => TaskResult::ok_tasks(InvocationId(0), reports),
                (false, _, _) => TaskResult::err_partial_tasks(
                    InvocationId(0),
                    reports,
                    ExpectedLen(expected_len),
                ),
            }
        }))
    }
}
