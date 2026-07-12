pub mod resolve_spec;

use crate::api::BsSystemApi;
use crate::run::resolve_spec::{InvokeRunTasks, ResolveSpec};
use crate::start::start_kind::run_from_input::RunFromInputPaths;
use crate::start::start_kind::StartKind;
use crate::start::start_system::DidStart;
use crate::start::SystemStart;
use crate::system::{BsSystem, CommitInput, ExternalEventMsg, ResolveInput, RunDryOk, RunOk};
use actix::{Actor, Addr};
use anyhow::Context;
use bsnext_core::shared_args::{InputOpts, LoggingOpts};
use bsnext_dto::any_event::AnyEvent;
use bsnext_dto::external_events::{ExternalEventsDTO, TaskTreePreview, TaskTreeSummary};
use bsnext_dto::StartupError;
use bsnext_input::route::{RunAll, RunOptItem, RunSeq, ShRunOptItem};
use bsnext_input::startup::{RunMode, StartupContext, SystemStartArgs, TopLevelRunMode};
use bsnext_input::{Input, InputError};
use bsnext_tracing::OutputFormat;
use std::path::PathBuf;
use std::time::Duration;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

fn about() -> String {
    let s = r#"
Use bslive to run groups of tasks and exit immediately after.
    "#;
    s.to_string()
}

#[derive(Debug, Clone, clap::Parser)]
#[command(about = about(), long_about = None)]
///
/// # Examples
/// Run a single command and exit immediately
///
/// ```rust
/// # use bsnext_system::cli::from_args_with_buffered_output;
/// # use bsnext_dto::external_events::has_output_line_matching;
/// # let rt = actix_rt::System::new();
/// # rt.block_on(async {
/// #   let args = r#"
/// bslive run --sh "echo 1"
/// # "#;
/// #   let words = shell_words::split(args).unwrap();
/// #   let cwd = std::path::PathBuf::from(std::env::current_dir().unwrap().to_string_lossy().to_string());
/// #   let (result, events) = from_args_with_buffered_output(words, cwd).await;
/// #   assert!(result.is_ok());
/// #   assert!(has_output_line_matching(&events, "1"));
/// # });
/// ```
///
pub struct RunCommand {
    /// commands to run
    pub trailing: Vec<String>,
    /// commands to run
    #[arg(long = "sh")]
    pub sh_commands: Vec<String>,
    /// provide this flag to disable command prefixes
    #[arg(long = "no-prefix", default_value = "false")]
    pub no_prefix: bool,
    /// logging options
    #[clap(flatten)]
    pub logging: LoggingOpts,
    /// run top-level tasks concurrently
    #[arg(long = "all")]
    pub all: bool,
    /// just print the task tree
    #[arg(long = "dry")]
    pub dry: bool,
    /// Whether to print the task tree before running
    /// This is different to '--dry' because this will still execute
    #[arg(long = "preview")]
    pub preview: bool,
    /// Whether to print the task tree summary after execution
    #[arg(long = "summary")]
    pub summary: bool,
    /// output format
    #[arg(short, long, value_enum, default_value_t)]
    pub format: OutputFormat,
}

#[derive(Debug)]
enum InputResolution {
    Default,
    UserDefined,
}

impl RunCommand {
    pub(crate) async fn prepare_input(
        &self,
        addr: &Addr<BsSystem>,
        input_opts: &InputOpts,
    ) -> anyhow::Result<(Input, InputResolution)> {
        // try to resolve input from disk
        let input = addr
            .send(ResolveInput::from_strs(&input_opts.input))
            .await
            .context("mailbox")??;

        let cmd_input = as_fake_input(&self);

        // otherwise, use a default input to start with...
        let (mut input, resolution) = match input {
            None => (cmd_input.clone(), InputResolution::Default),
            Some(mut resolved) => {
                resolved.input.run.extend(cmd_input.run.clone());
                (resolved.input, InputResolution::UserDefined)
            }
        };

        // todo: this shouldn't be needed, poor design found
        input.servers.clear();
        input.watchers.clear();

        Ok((input, resolution))
    }
}

impl SystemStart for RunCommand {
    fn resolve_input(&self, ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        todo!()
    }

    async fn start(
        &self,
        cwd: PathBuf,
        input_opts: InputOpts,
        sink: Sender<AnyEvent>,
    ) -> Result<DidStart, StartupError> {
        let (tx, rx) = oneshot::channel();
        let system = BsSystem::new(sink.clone(), cwd.clone(), tx);
        let addr = system.start();

        // prepare initial input before any tasks should run
        let (input, resolution) = self.prepare_input(&addr, &input_opts).await?;

        let named = if self.trailing.is_empty() {
            vec!["default".to_string()]
        } else {
            self.trailing.to_owned()
        };

        // dry takes precedence
        let run_mode = if self.dry {
            RunMode::Dry
        } else {
            RunMode::Exec {
                preview: self.preview,
                summary: self.summary,
            }
        };
        let top_level = TopLevelRunMode::Seq;

        let _id = input.as_id();
        let _r = addr.send(CommitInput::new(input.clone())).await;

        match run_mode {
            RunMode::Exec { .. } => {
                let _r2 =
                    run_jobs(&addr, input, named, top_level, self.preview, self.summary).await?;
            }
            RunMode::Dry => {
                let _d1 = print_jobs(&addr, input, named, top_level).await?;
            }
        }

        Ok(DidStart::WillExit)
    }
}

impl RunCommand {
    pub fn as_start_kind(&self, input_opts: &InputOpts) -> StartKind {
        let from_cmd = as_fake_input(self);

        tracing::debug!(self.trailing = ?self.trailing);
        tracing::debug!(self.sh_commands = ?self.sh_commands);
        tracing::debug!(self.all = ?self.all);

        let named = if self.trailing.is_empty() {
            vec!["default".to_string()]
        } else {
            self.trailing.to_owned()
        };

        // dry takes precedence
        let run_mode = if self.dry {
            RunMode::Dry
        } else {
            RunMode::Exec {
                preview: self.preview,
                summary: self.summary,
            }
        };
        let top_level = TopLevelRunMode::Seq;
        StartKind::Run(RunFromInputPaths::new(
            from_cmd,
            input_opts.input.clone(),
            named,
            run_mode,
            top_level,
        ))
    }
}

#[tracing::instrument]
fn as_fake_input(run: &RunCommand) -> Input {
    let mut input = Input::default();
    let mut list_of_commands: Vec<RunOptItem> = vec![];

    {
        for (index, sh) in run.sh_commands.iter().enumerate() {
            tracing::info!(index = index, sh = sh, name = "None", prefix = "None");
            list_of_commands.push(RunOptItem::Sh(ShRunOptItem {
                sh: sh.clone(),
                name: None,
                prefix: None,
            }));
        }
    }
    if list_of_commands.is_empty() {
        return input;
    };
    let mut items = vec![];
    if run.all {
        let run_all = RunAll::new(list_of_commands);
        items.push(RunOptItem::All(run_all));
    } else {
        let run_seq = RunSeq::new(list_of_commands);
        items.push(RunOptItem::Seq(run_seq));
    }
    input.run.insert("default".to_string(), items);
    input
}

async fn run_jobs(
    addr: &Addr<BsSystem>,
    input: Input,
    named: Vec<String>,
    top_level_run_mode: TopLevelRunMode,
    preview: bool,
    summary: bool,
) -> anyhow::Result<RunOk> {
    let spec_output = addr
        .send(ResolveSpec::new(input, named, top_level_run_mode))
        .await??;
    let spec = spec_output.as_spec();
    let tree = spec.as_tree();

    if preview {
        addr.send(ExternalEventMsg {
            evt: ExternalEventsDTO::TaskTreePreview(TaskTreePreview {
                tree: tree.clone(),
                will_exec: true,
            }),
        })
        .await??;
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    let report_and_tree = addr.send(InvokeRunTasks::new(spec.clone())).await??;

    if summary {
        let tree = spec.as_tree();
        addr.send(ExternalEventMsg {
            evt: ExternalEventsDTO::TaskTreeSummary(TaskTreeSummary::from_report(
                tree,
                &report_and_tree.report_map,
            )),
        })
        .await??;
    }

    Ok(RunOk { report_and_tree })
}

async fn print_jobs(
    addr: &Addr<BsSystem>,
    input: Input,
    named: Vec<String>,
    top_level_run_mode: TopLevelRunMode,
) -> anyhow::Result<RunDryOk> {
    let spec_output = addr
        .send(ResolveSpec::new(input, named, top_level_run_mode))
        .await??;
    let spec = spec_output.as_spec();
    let tree = spec.as_tree();
    addr.send(ExternalEventMsg {
        evt: ExternalEventsDTO::TaskTreePreview(TaskTreePreview {
            tree: tree.clone(),
            will_exec: false,
        }),
    })
    .await??;
    Ok(RunDryOk)
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::Parser;

    #[test]
    fn single_sh() -> anyhow::Result<()> {
        let run_cmd = RunCommand::try_parse_from(vec!["COMMAND_NAME", "--sh", "def"])?;
        assert_eq!(run_cmd.sh_commands.get(0), Some(&"def".to_string()));
        Ok(())
    }

    #[test]
    fn trailing() -> anyhow::Result<()> {
        let run_cmd = RunCommand::try_parse_from(vec!["COMMAND_NAME", "--preview", "default"])?;
        assert_eq!(run_cmd.trailing.get(0), Some(&"default".to_string()));
        assert_eq!(run_cmd.preview, true);
        Ok(())
    }
}
