use crate::api::BsSystemApi;
use crate::run::resolve_spec::InvokeRunTasks;
use crate::start::start_from_paths::{
    server_config_from_paths, with_explicit_paths, with_inferred_watchers,
};
use crate::start::start_system::DidStart;
use crate::start::SystemStart;
use crate::system::{BsSystem, CommitInput, CommitInputFile, GetStartContext, ResolveInput};
use crate::tasks::resolve::ResolveInitialTasks;
use crate::watch::watch_sub_opts::WatchSubOpts;
use actix::{Actor, Addr};
use anyhow::Context;
use bsnext_core::shared_args::{InputOpts, LoggingOpts};
use bsnext_dto::any_event::AnyEvent;
use bsnext_dto::StartupError;
use bsnext_input::route::{CorsOpts, Opts};
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use bsnext_input::startup::{StartupContext, SystemStartArgs};
use bsnext_input::{Input, InputArgs, InputCtx, InputError, WatchGlobalConfig};
use bsnext_tracing::OutputFormat;
use std::path::{Path, PathBuf};
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;

#[derive(Debug, Default, Clone, clap::Parser)]
pub struct StartCommand {
    /// Should permissive cors headers be added to all responses?
    #[arg(long)]
    pub cors: bool,

    /// Specify a port instead of a random one
    #[arg(short, long)]
    pub port: Option<u16>,

    #[arg(long = "proxy")]
    pub proxies: Vec<String>,

    /// logging options
    #[clap(flatten)]
    pub logging: LoggingOpts,

    /// output options
    #[arg(short, long, value_enum, default_value_t)]
    pub format: OutputFormat,

    /// Paths to serve + possibly watch, incompatible with `-i` option
    pub trailing: Vec<String>,

    /// disable all auto-watching
    #[clap(long)]
    pub no_watch: bool,

    /// additional watchers
    #[clap(flatten)]
    pub watch_sub_opts: WatchSubOpts,
}

impl SystemStart for StartCommand {
    fn resolve_input(&self, _ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        todo!()
    }

    async fn start(
        &self,
        cwd: PathBuf,
        input_opts: InputOpts,
        sink: Sender<AnyEvent>,
    ) -> Result<DidStart, StartupError> {
        // let fs_opts = self.fs_opts.clone();
        // let input_opts = self.input_opts.clone();
        let (tx, rx) = oneshot::channel();
        let system = BsSystem::new(sink.clone(), cwd.clone(), tx);
        let addr = system.start();

        let startup_ctx = addr
            .send(GetStartContext)
            .await
            .context("Trying to get ctx")?;

        // prepare initial input before any tasks should run
        let (mut input, resolution) = self.prepare_input(&addr, &input_opts).await?;

        // now run any 'before' tasks run before we try anything.
        initial_tasks(&addr, input.clone()).await?;

        if input.servers.is_empty() {
            let server_config = self.as_server_config(&cwd).await?;

            // add the server config
            input.servers.push(server_config);
        }

        // remember the ID of the input we are commiting
        let _id = input.as_id();
        match resolution {
            InputResolution::Default => {
                let _r = addr.send(CommitInput::new(input)).await;
            }
            InputResolution::File(file) => {
                let args = InputArgs::new(self.port);
                let ctx = InputCtx::new(&[], Some(args), &startup_ctx, Some(&file));
                let _r = addr.send(CommitInputFile::new(input, file, ctx)).await;
            }
        }

        let api = BsSystemApi::new(&addr, rx);
        Ok(DidStart::Started { api })
    }
}

enum InputResolution {
    Default,
    File(PathBuf),
}

impl StartCommand {
    async fn prepare_input(
        &self,
        addr: &Addr<BsSystem>,
        input_opts: &InputOpts,
    ) -> anyhow::Result<(Input, InputResolution)> {
        // try to resolve input from disk
        let input = addr
            .send(ResolveInput::from_strs(&input_opts.input))
            .await
            .context("mailbox")??;

        // otherwise, use a default input to start with...
        let (mut input, resolution) = match input {
            None => (Input::default(), InputResolution::Default),
            Some(resolved) => (resolved.input, InputResolution::File(resolved.absolute)),
        };

        if self.no_watch {
            input.config.watchers = WatchGlobalConfig::Disabled;
        } else {
            let explicit_watch_count =
                self.watch_sub_opts.paths.len() + self.watch_sub_opts.before.len();
            if explicit_watch_count > 0 {
                with_explicit_paths(&mut input, &self.watch_sub_opts);
            } else {
                with_inferred_watchers(&mut input, &self.watch_sub_opts);
            }
        };

        Ok((input, resolution))
    }

    async fn as_server_config(&self, cwd: &Path) -> anyhow::Result<ServerConfig> {
        let port = self.port;
        let paths = self.trailing.clone();
        let route_opts = Opts {
            cors: self.cors.then_some(CorsOpts::Cors(true)),
            ..Default::default()
        };

        let identity = ServerIdentity::from_port_or_named(port).context("port")?;

        let server_config = server_config_from_paths(cwd, &paths, &route_opts, identity)
            .context("server config")?;

        Ok(server_config)
    }
}

async fn initial_tasks(addr: &Addr<BsSystem>, input: Input) -> anyhow::Result<()> {
    let spec = addr.send(ResolveInitialTasks::new(input)).await??;
    let _report_and_tree = addr.send(InvokeRunTasks::new(spec)).await??;
    // let s = archy(&report_and_tree.tree, Prefix::None);
    Ok(())
}
