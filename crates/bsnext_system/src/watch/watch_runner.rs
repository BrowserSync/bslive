use bsnext_input::bs_live_built_in_task::BsLiveBuiltInTask;
use std::str::FromStr;

/// Representation for 'sh: echo hello world' and 'bslive:notify-server'
/// This allows CLI arguments
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum WatchRunnerStr {
    Sh(String),
    BsLive(BsLiveBuiltInTask),
}

impl FromStr for WatchRunnerStr {
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
    let result = WatchRunnerStr::from_str("sh: test command")?;
    assert_eq!(result, WatchRunnerStr::Sh("test command".to_string()));
    Ok(())
}

#[test]
fn test_watch_runner_from_str_bslive() -> anyhow::Result<()> {
    let result = WatchRunnerStr::from_str("bslive: notify-server")?;
    assert_eq!(
        result,
        WatchRunnerStr::BsLive(BsLiveBuiltInTask::NotifyServer)
    );
    Ok(())
}

#[test]
fn test_watch_sub_opts_parsing() -> anyhow::Result<()> {
    use crate::watch::watch_sub_opts::WatchSubOpts;
    use clap::Parser;

    let args = vec![
        "program",
        "--watch.paths",
        "tests",
        "--watch.run",
        "sh:echo 1",
    ];

    let opts = WatchSubOpts::try_parse_from(args)?;

    assert_eq!(opts.paths, vec!["tests"]);
    assert_eq!(opts.run, vec![WatchRunnerStr::Sh("echo 1".to_string())]);

    Ok(())
}
