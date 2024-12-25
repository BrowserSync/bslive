use crate::path_def::PathDef;
use crate::route::{DelayKind, DelayOpts, DirRoute, Opts, Route, RouteKind};
use clap::Parser;
use shell_words::split;

#[derive(clap::Parser, Debug)]
#[command(version)]
pub struct RouteCli {
    #[command(subcommand)]
    command: SubCommands,
}

impl RouteCli {
    pub fn try_from_cli_str<A: AsRef<str>>(a: A) -> Result<RouteCli, anyhow::Error> {
        let as_args = split(a.as_ref())?;
        RouteCli::try_parse_from(as_args).map_err(|e| anyhow::anyhow!(e))
    }
}

impl TryInto<Route> for RouteCli {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Route, Self::Error> {
        Ok(match self.command {
            SubCommands::ServeDir {
                dir,
                path,
                delay: Some(0),
            } => Route {
                path: PathDef::try_new(path)?,
                kind: RouteKind::Dir(DirRoute { dir, base: None }),
                ..std::default::Default::default()
            },
            SubCommands::ServeDir {
                dir,
                path,
                delay: ms,
            } => Route {
                path: PathDef::try_new(path)?,
                kind: RouteKind::Dir(DirRoute { dir, base: None }),
                opts: Opts {
                    delay: ms.map(|ms| DelayOpts::Delay(DelayKind::Ms(ms))),
                    ..std::default::Default::default()
                },
                ..std::default::Default::default()
            },
        })
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum SubCommands {
    /// does testing things
    ServeDir {
        /// Which path should this directory be served from
        #[arg(long, default_value = "/")]
        path: String,
        /// Which directory should be served
        #[arg(long, default_value = ".")]
        dir: String,

        #[arg(long)]
        delay: Option<u64>,
    },
}

#[cfg(test)]
mod test {
    use super::*;
    use clap::Parser;

    use shell_words::split;
    #[test]
    fn test_serve_dir() -> anyhow::Result<()> {
        let input = "bslive serve-dir --path=/ --dir=examples/basic/public";
        let as_args = split(input)?;
        let parsed = RouteCli::try_parse_from(as_args)?;
        let as_route: Route = parsed.try_into().unwrap();
        insta::assert_debug_snapshot!(as_route);
        Ok(())
    }
    #[test]
    fn test_serve_dir_delay() -> anyhow::Result<()> {
        let input = "bslive serve-dir --path=/ --dir=examples/basic/public --delay=1000";
        let as_args = split(input)?;
        let parsed = RouteCli::try_parse_from(as_args)?;
        let as_route: Route = parsed.try_into().unwrap();
        insta::assert_debug_snapshot!(as_route);
        Ok(())
    }
}
