use bsnext_input::route::{
    BeforeRunOptItem, MultiWatch, PrefixOpt, RunOptItem, ShRunOptItem, Spec, WatcherDirs,
};

#[derive(Debug, Clone, clap::Parser)]
pub struct WatchCommand {
    /// Paths to watch
    #[arg(required = true)]
    pub paths: Vec<String>,
    /// Commands to run when files have changed
    #[arg(long = "command", short)]
    pub command: Vec<String>,
    /// Initial command to run before starting watchers
    #[arg(long = "initial", short, default_value = "false")]
    pub initial: bool,
    /// provide this flag to disable command prefixes
    #[arg(long = "no-prefix", default_value = "false")]
    pub no_prefix: bool,
}

impl From<WatchCommand> for MultiWatch {
    fn from(value: WatchCommand) -> Self {
        let dirs = WatcherDirs::Many(value.paths);
        let run_opts = value
            .command
            .iter()
            .map(ToOwned::to_owned)
            .enumerate()
            .map(move |(index, item)| {
                let name = Some(format!("command:{}", index));
                let prefix = value.no_prefix.then_some(PrefixOpt::Bool(false));
                RunOptItem::Sh(ShRunOptItem {
                    sh: item,
                    name,
                    prefix,
                })
            })
            .collect::<Vec<_>>();

        let before = value.initial.then(|| {
            let cloned = run_opts.clone();
            cloned
                .into_iter()
                .map(|old| match old {
                    RunOptItem::Sh(old) => {
                        let name = Some("initial".to_string());
                        let prefix = value.no_prefix.then_some(PrefixOpt::Bool(false));
                        BeforeRunOptItem::Sh(ShRunOptItem {
                            sh: old.sh,
                            name,
                            prefix,
                        })
                    }
                    _ => unreachable!(),
                })
                .collect::<Vec<_>>()
        });

        let spec = Spec {
            before,
            run: Some(run_opts),
            ..Default::default()
        };

        MultiWatch {
            dirs,
            opts: Some(spec),
        }
    }
}
