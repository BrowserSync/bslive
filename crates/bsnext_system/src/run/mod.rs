use bsnext_core::shared_args::LoggingOpts;
use bsnext_input::route::{RunAll, RunOptItem, RunSeq, ShRunOptItem};
use bsnext_input::Input;
use bsnext_tracing::OutputFormat;
use tracing::instrument;

#[derive(Debug, Clone, clap::Parser)]
pub struct RunCommand {
    /// commands to run
    pub sh_commands_implicit: Vec<String>,
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
            let len = self.sh_commands.len();
            for (index, sh) in self.sh_commands.iter().enumerate() {
                tracing::info!(index = index, sh = sh);
                let named = if len == 1 {
                    "[sh]".to_string()
                } else {
                    format!("[sh: {index}]")
                };
                list_of_commands.push(RunOptItem::Sh(ShRunOptItem {
                    sh: sh.clone(),
                    name: Some(named),
                    prefix: None,
                }));
            }
        }
        {
            let span = tracing::debug_span!("adding sh_commands commands from cli");
            let _enter = span.enter();
            let len = self.sh_commands_implicit.len();
            for (index, sh) in self.sh_commands_implicit.iter().enumerate() {
                tracing::info!(index = index, sh = sh);
                let named = if len == 1 {
                    "[sh-trailing]".to_string()
                } else {
                    format!("[sh-trailing: {index}]")
                };
                list_of_commands.push(RunOptItem::Sh(ShRunOptItem {
                    sh: sh.clone(),
                    name: Some(named),
                    prefix: None,
                }));
            }
        }

        if self.all {
            let run_all = RunAll::new(list_of_commands);
            input.run.push(RunOptItem::All(run_all));
        } else {
            let run_seq = RunSeq::new(list_of_commands);
            input.run.push(RunOptItem::Seq(run_seq));
        }

        input
    }
}
