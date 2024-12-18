use crate::path_def::PathDef;
use crate::route::{DirRoute, Route, RouteKind};
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
            SubCommands::ServeDir { dir, path } => Route {
                path: PathDef::try_new(path)?,
                kind: RouteKind::Dir(DirRoute { dir, base: None }),
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
        #[arg(short, long, default_value = "/")]
        path: String,
        /// Which directory should be served
        #[arg(short, long, default_value = ".")]
        dir: String,
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
        let as_route: Result<Route, _> = parsed.try_into();
        dbg!(&as_route);
        // assert_debug_snapshot!(parsed);
        Ok(())
    }
}
