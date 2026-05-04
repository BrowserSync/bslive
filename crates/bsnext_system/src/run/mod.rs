pub mod resolve_spec;

use crate::start::start_kind::run_from_input::RunFromInputPaths;
use crate::start::start_kind::StartKind;
use bsnext_core::shared_args::{InputOpts, LoggingOpts};
use bsnext_input::route::{RunAll, RunOptItem, RunSeq, ShRunOptItem};
use bsnext_input::startup::{RunMode, TopLevelRunMode};
use bsnext_input::Input;
use bsnext_tracing::OutputFormat;

#[derive(Debug, Clone, clap::Parser)]
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

impl RunCommand {
    pub fn as_start_kind(&self, input_opts: &InputOpts) -> StartKind {
        let from_cmd = as_input(self);

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
fn as_input(run: &RunCommand) -> Input {
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

    #[test]
    fn input() -> anyhow::Result<()> {
        let run_cmd = RunCommand::try_parse_from(vec!["COMMAND_NAME", "--sh", "def"])?;
        let as_input = run_cmd.as_input();
        let s = vec![RunOptItem::Seq(RunSeq::new(vec![RunOptItem::Sh(
            ShRunOptItem::new("def"),
        )]))];
        assert_eq!(as_input.run.get("default"), Some(&s));
        Ok(())
    }
}
