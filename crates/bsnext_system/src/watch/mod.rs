use bsnext_core::shared_args::LoggingOpts;
use bsnext_input::bs_live_built_in_task::BsLiveBuiltInTask;
use bsnext_input::route::{
    BeforeRunOptItem, MultiWatch, PrefixOpt, RunOptItem, ShRunOptItem, Spec, WatcherDirs,
};
use bsnext_tracing::OutputFormat;
use std::str::FromStr;

#[derive(Debug, Clone, clap::Parser)]
pub struct WatchCommand {
    /// Paths to watch
    #[arg(required = true)]
    pub paths: Vec<String>,
    /// sh Commands to run when files have changed
    #[arg(long = "sh")]
    pub sh_commands: Vec<String>,
    /// Initial command to run before starting watchers
    #[arg(long = "initial", short)]
    pub initial: Vec<String>,
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

#[derive(Debug, Default, Clone, clap::Parser)]
pub struct WatchSubOpts {
    #[arg(long = "watch.path", num_args(0..))]
    pub paths: Vec<String>,
    #[arg(long = "watch.run", num_args(0..))]
    pub run: Vec<WatchRunner>,
}

impl WatchSubOpts {
    pub fn sh_commands(&self) -> Vec<String> {
        self.run
            .iter()
            .filter_map(|run| match run {
                WatchRunner::Sh(sh) => Some(sh.to_owned()),
                WatchRunner::BsLive(_) => None,
            })
            .collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WatchRunner {
    Sh(String),
    BsLive(BsLiveBuiltInTask),
}

impl FromStr for WatchRunner {
    type Err = anyhow::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.split_once(":") {
            Some(("sh", str)) => Ok(Self::Sh(str.trim().to_string())),
            Some(("bslive", str)) => Ok(Self::BsLive(BsLiveBuiltInTask::from_str(str.trim())?)),
            _ => Err(anyhow::anyhow!("not supported")),
        }
    }
}

#[test]
fn test_watch_runner_from_str() -> anyhow::Result<()> {
    let result = WatchRunner::from_str("sh: test command")?;
    assert_eq!(result, WatchRunner::Sh("test command".to_string()));
    Ok(())
}

#[test]
fn test_watch_runner_from_str_bslive() -> anyhow::Result<()> {
    let result = WatchRunner::from_str("bslive: notify-server")?;
    assert_eq!(result, WatchRunner::BsLive(BsLiveBuiltInTask::NotifyServer));
    Ok(())
}

#[test]
fn test_watch_sub_opts_parsing() -> anyhow::Result<()> {
    use clap::Parser;

    let args = vec![
        "program",
        "--watch.path",
        "tests",
        "--watch.run",
        "sh:echo 1",
    ];

    let opts = WatchSubOpts::try_parse_from(args)?;

    assert_eq!(opts.paths, vec!["tests"]);
    assert_eq!(opts.run, vec![WatchRunner::Sh("echo 1".to_string())]);

    Ok(())
}

impl From<WatchSubOpts> for WatchCommand {
    fn from(value: WatchSubOpts) -> Self {
        WatchCommand {
            paths: value.paths.clone(),
            sh_commands: value.sh_commands(),
            initial: vec![],
            no_prefix: false,
            logging: Default::default(),
            format: Default::default(),
        }
    }
}

impl From<WatchSubOpts> for MultiWatch {
    fn from(value: WatchSubOpts) -> Self {
        let dirs = WatcherDirs::Many(value.paths);
        let run_opts = value
            .run
            .iter()
            .map(ToOwned::to_owned)
            .enumerate()
            .map(move |(index, item)| {
                let _name = Some(format!("command:{index}"));
                match item {
                    WatchRunner::Sh(sh) => {
                        let name = Some(format!("command:{index}"));
                        RunOptItem::Sh(ShRunOptItem {
                            sh,
                            name,
                            prefix: None,
                        })
                    }
                    WatchRunner::BsLive(bslive) => RunOptItem::BsLive { bslive },
                }
            })
            .collect::<Vec<_>>();
        let spec = Spec {
            before: None,
            run: Some(run_opts),
            ..Default::default()
        };
        MultiWatch {
            dirs,
            opts: Some(spec),
        }
    }
}

impl From<WatchCommand> for MultiWatch {
    fn from(value: WatchCommand) -> Self {
        let dirs = WatcherDirs::Many(value.paths);
        let run_opts = value
            .sh_commands
            .iter()
            .map(ToOwned::to_owned)
            .enumerate()
            .map(move |(index, item)| {
                let name = Some(format!("command:{index}"));
                let prefix = value.no_prefix.then_some(PrefixOpt::Bool(false));
                RunOptItem::Sh(ShRunOptItem {
                    sh: item,
                    name,
                    prefix,
                })
            })
            .collect::<Vec<_>>();

        let before = value
            .initial
            .iter()
            .map(ToOwned::to_owned)
            .enumerate()
            .map(move |(index, item)| {
                let name = Some(format!("initial:{index}"));
                let prefix = value.no_prefix.then_some(PrefixOpt::Bool(false));
                BeforeRunOptItem::Sh(ShRunOptItem {
                    sh: item,
                    name,
                    prefix,
                })
            })
            .collect::<Vec<_>>();

        let spec = Spec {
            before: Some(before),
            run: Some(run_opts),
            ..Default::default()
        };

        MultiWatch {
            dirs,
            opts: Some(spec),
        }
    }
}
