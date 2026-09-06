use crate::api::BsSystemApi;
use crate::run::resolve_spec::InvokeRunTasks;
use crate::start::start_system::DidStart;
use crate::start::SystemStart;
use crate::system::{BsSystem, CommitInput, ResolveInput};
use crate::tasks::resolve::ResolveInitialTasks;
use crate::watch::watch_sub_opts::WatchSubOpts;
use actix::{Actor, Addr};
use anyhow::Context;
use bsnext_core::shared_args::{InputOpts, LoggingOpts};
use bsnext_dto::any_event::AnyEvent;
use bsnext_dto::StartupError;
use bsnext_input::route::{MultiWatch, PathPattern};
use bsnext_input::startup::{StartupContext, SystemStartArgs};
use bsnext_input::{Input, InputError};
use bsnext_tracing::OutputFormat;
use std::path::PathBuf;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use watch_runner::WatchRunnerStr;

pub mod watch_runner;
pub mod watch_sub_opts;

#[derive(Debug, Default, Clone, clap::Parser)]
pub struct WatchCommand {
    /// Paths to watch
    #[arg(required = true)]
    pub paths: Vec<String>,
    #[arg(long, num_args(0..))]
    pub before: Vec<WatchRunnerStr>,
    /// sh Commands to run when files have changed
    #[arg(long, num_args(0..))]
    pub run: Vec<WatchRunnerStr>,
    /// if true, listed commands will execute once before watching starts
    #[arg(long)]
    pub initial: bool,
    /// how long to buffer changes for
    #[arg(long)]
    pub debounce: Option<usize>,
    /// paths to ignore
    #[arg(long, num_args(0..))]
    pub ignore: Vec<PathPattern>,
    /// patterns to allow - when given, paths MUST match one of these
    #[arg(long, num_args(0..))]
    pub only: Vec<PathPattern>,
    /// provide this flag to disable command prefixes
    #[arg(long = "no-prefix", default_value = "false")]
    pub no_prefix: bool,
    /// logging options
    #[clap(flatten)]
    pub logging: LoggingOpts,
    /// output format
    #[arg(short, long, value_enum, default_value_t)]
    pub format: OutputFormat,
}

impl WatchCommand {
    pub(crate) async fn prepare_input(
        &self,
        addr: &Addr<BsSystem>,
        input_opts: &InputOpts,
    ) -> anyhow::Result<Input> {
        // try to resolve input from disk
        let input = addr
            .send(ResolveInput::from_strs(&input_opts.input))
            .await
            .context("mailbox")??;

        let mut input = match input {
            None => Input::default(),
            Some(resolved) => {
                // resolved.input.run.extend(input.run.clone());
                // (resolved.input, InputResolution::UserDefined)
                resolved.input
            }
        };

        let multi = MultiWatch::from(self.clone());
        input.watchers.push(multi);

        // todo: this should be better;
        input.servers.clear();

        Ok(input)
    }
}

impl SystemStart for WatchCommand {
    fn resolve_input(&self, _ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
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
        let input = self.prepare_input(&addr, &input_opts).await?;

        // now run any 'before' tasks run before we try anything.
        initial_tasks(&addr, input.clone()).await?;

        // remember the ID of the input we are commiting
        let _id = input.as_id();
        let _r = addr.send(CommitInput::new(input)).await;

        let api = BsSystemApi::new(&addr, rx);
        Ok(DidStart::Started { api })
    }
}

async fn initial_tasks(addr: &Addr<BsSystem>, input: Input) -> anyhow::Result<()> {
    let spec = addr.send(ResolveInitialTasks::new(input)).await??;
    let _report_and_tree = addr.send(InvokeRunTasks::new(spec)).await??;
    // let s = archy(&report_and_tree.tree, Prefix::None);
    Ok(())
}

impl From<WatchCommand> for MultiWatch {
    fn from(value: WatchCommand) -> Self {
        let sub_opts = WatchSubOpts {
            paths: value.paths,
            run: value.run,
            before: value.before,
            ignore: value.ignore,
            only: value.only,
            initial: value.initial,
            debounce: value.debounce,
        };
        MultiWatch::from(sub_opts)
    }
}
