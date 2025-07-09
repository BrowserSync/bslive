use crate::path_def::PathDef;
use crate::route::{CorsOpts, DelayKind, DelayOpts, DirRoute, Opts, ProxyRoute, Route, RouteKind};
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
                dir, path, opts, ..
            } => Route {
                path: PathDef::try_new(path)?,
                kind: RouteKind::Dir(DirRoute { dir, base: None }),
                opts: opts_to_route_opts(&opts),
                ..std::default::Default::default()
            },
            SubCommands::Proxy { path, target, opts } => Route {
                path: PathDef::try_new(path)?,
                kind: RouteKind::Proxy(ProxyRoute {
                    proxy: target,
                    proxy_headers: None,
                    rewrite_uri: None,
                }),
                opts: opts_to_route_opts(&opts),
                ..std::default::Default::default()
            },
        })
    }
}

fn opts_to_route_opts(cli_opts: &RouteCliSharedOpts) -> Opts {
    Opts {
        delay: cli_opts.delay.map(|ms| DelayOpts::Delay(DelayKind::Ms(ms))),
        cors: cli_opts.cors.map(CorsOpts::Cors),
        ..std::default::Default::default()
    }
}

#[derive(Debug, clap::Subcommand)]
pub enum SubCommands {
    /// does testing things
    Proxy {
        #[arg(long, default_value = "/proxy")]
        path: String,
        #[arg(long)]
        target: String,
        #[clap(flatten)]
        opts: RouteCliSharedOpts,
    },
    ServeDir {
        /// Which path should this directory be served from
        #[arg(long, default_value = "/")]
        path: String,
        /// Which directory should be served
        #[arg(long, default_value = ".")]
        dir: String,
        #[clap(flatten)]
        opts: RouteCliSharedOpts,
    },
}

#[derive(Debug, clap::Parser)]
pub struct RouteCliSharedOpts {
    /// should a ms delay be applied?
    #[arg(long)]
    delay: Option<u64>,
    /// should cors headers be added to responses
    #[arg(long)]
    cors: Option<bool>,
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
    #[test]
    fn test_serve_dir_shared() -> anyhow::Result<()> {
        let input =
            "bslive serve-dir --path=/ --dir=examples/basic/public --delay=1000 --cors=true";
        let as_args = split(input)?;
        let parsed = RouteCli::try_parse_from(as_args)?;
        let as_route: Route = parsed.try_into().unwrap();
        insta::assert_debug_snapshot!(as_route);
        Ok(())
    }
}
