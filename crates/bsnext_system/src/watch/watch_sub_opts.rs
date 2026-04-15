use crate::watch::watch_runner::WatchRunnerStr;
use bsnext_input::route::{
    BeforeRunOptItem, MultiWatch, RunOptItem, ShRunOptItem, Spec, WatcherDirs,
};

#[derive(Debug, Default, Clone, clap::Parser)]
pub struct WatchSubOpts {
    #[arg(long = "watch.paths", num_args(0..))]
    pub paths: Vec<String>,
    #[arg(long = "watch.run", num_args(0..))]
    pub run: Vec<WatchRunnerStr>,
    #[arg(long = "watch.before", num_args(0..))]
    pub before: Vec<WatchRunnerStr>,
    #[arg(long = "watch.initial")]
    pub initial: bool,
}

impl From<WatchSubOpts> for MultiWatch {
    fn from(value: WatchSubOpts) -> Self {
        let dirs = WatcherDirs::Many(value.paths);
        let run_opts = value
            .run
            .iter()
            .map(ToOwned::to_owned)
            .map(move |item| match item {
                WatchRunnerStr::Sh(sh) => RunOptItem::Sh(ShRunOptItem {
                    sh,
                    name: None,
                    prefix: None,
                }),
                WatchRunnerStr::BsLive(bslive) => RunOptItem::BsLive { bslive },
            })
            .collect::<Vec<_>>();

        let initial = value.initial.then(|| {
            run_opts
                .iter()
                .map(ToOwned::to_owned)
                .filter_map(move |item| match item {
                    RunOptItem::Sh(sh) => Some(BeforeRunOptItem::Sh(sh)),
                    _ => None,
                })
                .collect::<Vec<_>>()
        });

        let before_explicit = (!value.before.is_empty()).then(|| {
            value
                .before
                .iter()
                .filter_map(|watch| match watch {
                    WatchRunnerStr::Sh(sh) => Some(BeforeRunOptItem::Sh(ShRunOptItem {
                        sh: sh.to_owned(),
                        name: None,
                        prefix: None,
                    })),
                    _ => None,
                })
                .collect::<Vec<_>>()
        });

        // if 'before' was given too, always run that first
        let queue: Option<Vec<BeforeRunOptItem>> = match (initial, before_explicit) {
            (Some(initial), Some(before)) => Some(before.into_iter().chain(initial).collect()),
            (None, Some(before)) => Some(before),
            (Some(initial), None) => Some(initial),
            (None, None) => None,
        };

        let spec = Spec {
            before: queue,
            run: Some(run_opts),
            ..Default::default()
        };
        MultiWatch {
            dirs,
            opts: Some(spec),
        }
    }
}
