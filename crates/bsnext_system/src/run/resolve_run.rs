use crate::tasks::task_spec::TaskSpec;
use crate::BsSystem;
use actix::{AsyncContext, ResponseFuture};
use bsnext_dto::internal::{Available, Expected, InitialTaskError, TaskReportAndTree};
use bsnext_input::route::RunOptItem;
use bsnext_input::startup::TopLevelRunMode;
use bsnext_input::Input;
use std::collections::HashMap;

#[derive(actix::Message)]
#[rtype(result = "Result<ResolveRunTasksOutput, InitialTaskError>")]
pub struct ResolveRunTasks {
    input: Input,
    named: Vec<String>,
    top_level_run_mode: TopLevelRunMode,
}

pub struct ResolveRunTasksOutput {
    pub task_spec: TaskSpec,
}

impl ResolveRunTasks {
    pub fn new(input: Input, named: Vec<String>, top_level_run_mode: TopLevelRunMode) -> Self {
        Self {
            input,
            named,
            top_level_run_mode,
        }
    }
}

impl actix::Handler<ResolveRunTasks> for BsSystem {
    type Result = ResponseFuture<Result<ResolveRunTasksOutput, InitialTaskError>>;

    #[tracing::instrument(skip_all, name = "ResolveRunTasks")]
    fn handle(&mut self, msg: ResolveRunTasks, ctx: &mut Self::Context) -> Self::Result {
        let _addr = ctx.address();
        tracing::debug!(run.lookup.keys = ?msg.named);
        tracing::debug!(run.lookup.available = ?msg.input.run.keys());
        #[derive(Debug)]
        enum Match<'a> {
            Ok { named: &'a Vec<RunOptItem> },
            Missing,
        }

        let matched: HashMap<&str, Match> = msg
            .named
            .iter()
            .map(|name| {
                let v = match msg.input.run.get(name) {
                    None => Match::Missing,
                    Some(named) => Match::Ok { named },
                };
                (name.as_str(), v)
            })
            .collect();

        tracing::debug!(run.lookup.matched = ?matched);

        let (bad, _good): (Vec<_>, Vec<_>) = matched
            .iter()
            .partition(|(_named, matcher)| matches!(matcher, Match::Missing));

        if !bad.is_empty() {
            let bad = bad
                .into_iter()
                .map(|(x, _i)| x.to_string())
                .collect::<Vec<_>>();
            let available = msg
                .input
                .run
                .keys()
                .map(ToOwned::to_owned)
                .collect::<Vec<_>>();
            // let bad = bad.into_iter().map(|(x, i)| x.to_string()).collect();
            return Box::pin(async move {
                Err(InitialTaskError::MissingTask {
                    expected: Expected(bad),
                    available: Available(available),
                })
            });
        }

        for (named, matched) in &matched {
            match matched {
                Match::Ok { .. } => {}
                Match::Missing => {
                    tracing::error!("missing run item: {}", named);
                }
            }
        }

        let ordered = matched
            .iter()
            .filter_map(|(_name, matched)| match matched {
                Match::Ok { named } => Some(*named),
                Match::Missing => None,
            })
            .flatten()
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();

        tracing::debug!(run.lookup.ordered = ?ordered);

        let spec = match msg.top_level_run_mode {
            TopLevelRunMode::Seq => TaskSpec::seq_from(&ordered),
            TopLevelRunMode::All => TaskSpec::all_from(&ordered),
        };

        Box::pin(async move { Ok(ResolveRunTasksOutput { task_spec: spec }) })
    }
}

#[derive(actix::Message)]
#[rtype(result = "Result<TaskReportAndTree, InitialTaskError>")]
pub struct InvokeRunTasks {
    task_spec: TaskSpec,
}

impl InvokeRunTasks {
    pub fn new(task_spec: TaskSpec) -> Self {
        Self { task_spec }
    }
}

impl actix::Handler<InvokeRunTasks> for BsSystem {
    type Result = ResponseFuture<Result<TaskReportAndTree, InitialTaskError>>;

    #[tracing::instrument(skip_all, name = "ResolveRunTasks")]
    fn handle(&mut self, msg: InvokeRunTasks, ctx: &mut Self::Context) -> Self::Result {
        let addr = ctx.address();
        let (invoke_scope, rx) = self.run_only(addr, msg.task_spec);
        ctx.notify(invoke_scope);
        Box::pin(async move {
            match rx.await {
                Ok(TaskReportAndTree { report, tree }) if report.is_ok() => {
                    Ok(TaskReportAndTree { report, tree })
                }
                Ok(TaskReportAndTree { .. }) => Err(InitialTaskError::FailedReport),
                Err(_) => Err(InitialTaskError::FailedUnknown),
            }
        })
    }
}
