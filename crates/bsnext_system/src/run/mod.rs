pub mod resolve_run;

use bsnext_core::shared_args::LoggingOpts;
use bsnext_input::route::{RunAll, RunOptItem, RunSeq, ShRunOptItem};
use bsnext_input::Input;
use bsnext_tracing::OutputFormat;
use tracing::instrument;

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
    /// output format
    #[arg(short, long, value_enum, default_value_t)]
    pub format: OutputFormat,
}

impl RunCommand {
    #[instrument(name = "as_input", skip(self))]
    pub fn as_input(&self) -> Input {
        let mut input = Input::default();
        let mut list_of_commands: Vec<RunOptItem> = vec![];

        {
            let span = tracing::debug_span!("adding sh_commands commands from cli");
            let _enter = span.enter();
            for (index, sh) in self.sh_commands.iter().enumerate() {
                tracing::info!(index = index, sh = sh);
                list_of_commands.push(RunOptItem::Sh(ShRunOptItem {
                    sh: sh.clone(),
                    name: None,
                    prefix: None,
                }));
            }
        }

        let mut items = vec![];
        if self.all {
            let run_all = RunAll::new(list_of_commands);
            items.push(RunOptItem::All(run_all));
        } else {
            let run_seq = RunSeq::new(list_of_commands);
            items.push(RunOptItem::Seq(run_seq));
        }
        input.run.insert("default".to_string(), items);

        input
    }
}
