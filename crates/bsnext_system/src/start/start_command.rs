use crate::start::start_kind::start_from_inputs::{StartFromInput, StartFromInputPaths};
use crate::start::start_kind::start_from_paths::StartFromPaths;
use crate::start::start_kind::StartKind;
use crate::watch::watch_sub_opts::WatchSubOpts;
use bsnext_core::shared_args::{FsOpts, InputOpts, LoggingOpts};
use bsnext_input::route::{CorsOpts, Opts, Route};
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use bsnext_input::Input;
use bsnext_tracing::OutputFormat;

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

impl StartCommand {
    pub fn as_start_kind(&self, fs_opts: &FsOpts, input_opts: &InputOpts) -> StartKind {
        // todo: make the addition of a proxy + route opts easier?
        if !self.trailing.is_empty() {
            tracing::debug!(
                "{} trailing, {} inputs",
                self.trailing.len(),
                input_opts.input.len()
            );
            return StartKind::FromPaths(StartFromPaths {
                paths: self.trailing.clone(),
                write_input: fs_opts.write,
                port: self.port,
                force: fs_opts.force,
                watch_sub_opts: self.watch_sub_opts.clone(),
                route_opts: Opts {
                    cors: self.cors.then_some(CorsOpts::Cors(true)),
                    ..Default::default()
                },
                no_watch: self.no_watch,
            });
        }

        tracing::debug!("0 trailing, {} inputs", input_opts.input.len());
        if input_opts.input.is_empty() && !self.proxies.is_empty() {
            tracing::debug!("input was empty, but had proxies");
            let first_proxy = self.proxies.first().expect("guarded first proxy");
            let r = Route::proxy(first_proxy);
            let id = ServerIdentity::from_port_or_named(self.port).unwrap_or_else(|_e| {
                tracing::error!("A problem occurred with the port?");
                ServerIdentity::named()
            });
            let ser = ServerConfig::from_route(r, id);
            let input = Input::from_server(ser);
            StartKind::FromInput(StartFromInput { input })
        } else {
            tracing::debug!(
                input_len = input_opts.input.len(),
                proxes = self.proxies.len(),
                "neither inputs nor proxies were present"
            );
            StartKind::FromInputPaths(StartFromInputPaths {
                input_paths: input_opts.input.clone(),
                port: self.port,
                no_watch: self.no_watch,
            })
        }
    }
}
