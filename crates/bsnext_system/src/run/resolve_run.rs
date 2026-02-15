use crate::tasks::task_spec::TaskSpec;
use crate::BsSystem;
use actix::{AsyncContext, ResponseFuture};
use bsnext_dto::internal::{Available, Expected, InitialTaskError, TaskReportAndTree};
use bsnext_input::route::RunOptItem;
use bsnext_input::Input;
use std::collections::HashMap;

#[derive(actix::Message)]
#[rtype(result = "Result<TaskReportAndTree, InitialTaskError>")]
pub struct ResolveRunTasks {
    input: Input,
    named: Vec<String>,
}

impl ResolveRunTasks {
    pub fn new(input: Input, named: Vec<String>) -> Self {
        Self { input, named }
    }
}

impl actix::Handler<ResolveRunTasks> for BsSystem {
    type Result = ResponseFuture<Result<TaskReportAndTree, InitialTaskError>>;

    #[tracing::instrument(skip_all, name = "ResolveRunTasks")]
    fn handle(&mut self, msg: ResolveRunTasks, ctx: &mut Self::Context) -> Self::Result {
        let addr = ctx.address();
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

        let spec = TaskSpec::seq_from(&ordered);
        let (invoke_scope, rx) = self.run_only(addr, spec);
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
