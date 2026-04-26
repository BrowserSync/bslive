use crate::watch::watch_runner::WatchRunnerStr;
use bsnext_input::route::{
    BeforeRunOptItem, DebounceDuration, MultiWatch, PathPattern, RunOptItem, ShRunOptItem,
    WatchSpec, WatcherDirs,
};

#[derive(Debug, Default, Clone, clap::Parser)]
pub struct WatchSubOpts {
    #[arg(long = "watch.paths", num_args(0..))]
    pub paths: Vec<String>,
    #[arg(long = "watch.run", num_args(0..))]
    pub run: Vec<WatchRunnerStr>,
    #[arg(long = "watch.before", num_args(0..))]
    pub before: Vec<WatchRunnerStr>,
    #[arg(long = "watch.ignore", num_args(0..))]
    pub ignore: Vec<PathPattern>,
    #[arg(long = "watch.only", num_args(0..))]
    pub only: Vec<PathPattern>,

    // milliseconds debounce
    #[arg(long = "watch.debounce")]
    pub debounce: Option<usize>,

    #[arg(long = "watch.initial")]
    pub initial: bool,
}

impl WatchSubOpts {
    pub fn empty() -> Self {
        Self::default()
    }
}

impl From<WatchSubOpts> for MultiWatch {
    fn from(value: WatchSubOpts) -> Self {
        let span = tracing::debug_span!("creating MultiWatch");
        let _g = span.entered();
        let dirs = WatcherDirs::Many(value.paths);

        // first, capture '--run' args as explicit commands
        let run_opts = (!value.run.is_empty()).then(|| {
            value
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
                .collect::<Vec<_>>()
        });

        // if '--initial' was provided, mirror the `--run` list as something to run initially.
        let initial_mirror = match (value.initial, &run_opts) {
            (true, Some(opts)) => Some(
                opts.iter()
                    .map(ToOwned::to_owned)
                    .filter_map(move |item| match item {
                        RunOptItem::Sh(sh) => Some(BeforeRunOptItem::Sh(sh)),
                        _ => None,
                    })
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        };

        // if `--before=...` commands were given
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
        let queue: Option<Vec<BeforeRunOptItem>> = match (initial_mirror, before_explicit) {
            (Some(initial), Some(before)) => Some(before.into_iter().chain(initial).collect()),
            (None, Some(before)) => Some(before),
            (Some(initial), None) => Some(initial),
            (None, None) => None,
        };

        let ignore = (!value.ignore.is_empty()).then(|| {
            tracing::trace!(
                spec.ignore.len = value.ignore.len(),
                "adding global ignore patterns"
            );
            PathPattern::List(value.ignore)
        });

        let only = (!value.only.is_empty()).then(|| {
            tracing::trace!(
                spec.ignore.len = value.only.len(),
                "adding global only patterns"
            );
            PathPattern::List(value.only)
        });

        let debounce = value
            .debounce
            .map(|duration| DebounceDuration::Ms(duration as u64));

        let spec = WatchSpec {
            before: queue,
            run: run_opts,
            debounce,
            only,
            ignore,
        };

        MultiWatch {
            dirs,
            spec: Some(spec),
        }
    }
}
