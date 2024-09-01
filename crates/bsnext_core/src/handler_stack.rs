use crate::handler_stack;
use bsnext_input::route::{DirRoute, ProxyRoute, RawRoute, Route, RouteKind};

#[derive(Debug, PartialEq)]
pub enum HandlerStack {
    None,
    // todo: make this a separate thing
    Raw(RawRoute),
    Dirs(Vec<DirRoute>),
    Proxy(ProxyRoute),
    DirsProxy(Vec<DirRoute>, ProxyRoute),
}

pub fn routes_to_stack(state: HandlerStack, route: Route) -> HandlerStack {
    match state {
        HandlerStack::None => match route.kind {
            RouteKind::Raw(route) => HandlerStack::Raw(route),
            RouteKind::Proxy(pr) => HandlerStack::Proxy(pr),
            RouteKind::Dir(dir) => HandlerStack::Dirs(vec![dir]),
        },
        HandlerStack::Raw(..) => match route.kind {
            // if a second 'raw' is seen, just use it, discarding the previous
            RouteKind::Raw(route) => HandlerStack::Raw(route),
            // 'raw' handlers never get updated
            _ => state,
        },
        HandlerStack::Dirs(mut dirs) => match route.kind {
            RouteKind::Dir(next_dir) => {
                dirs.push(next_dir);
                HandlerStack::Dirs(dirs)
            }
            RouteKind::Proxy(proxy) => HandlerStack::DirsProxy(dirs, proxy),
            _ => HandlerStack::Dirs(dirs),
        },
        HandlerStack::Proxy(proxy) => match route.kind {
            RouteKind::Dir(dir) => HandlerStack::DirsProxy(vec![dir], proxy),
            _ => HandlerStack::Proxy(proxy),
        },
        HandlerStack::DirsProxy(mut dirs, proxy) => match route.kind {
            RouteKind::Dir(dir) => {
                dirs.push(dir);
                HandlerStack::DirsProxy(dirs, proxy)
            }
            _ => HandlerStack::DirsProxy(dirs, proxy),
        },
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::handler_stack::HandlerStack::{Dirs, DirsProxy};
    use bsnext_input::Input;
    #[test]
    fn test_handler_stack_01() -> anyhow::Result<()> {
        let yaml = include_str!("../../../examples/basic/handler_stack.yml");
        let input = serde_yaml::from_str::<Input>(&yaml)?;

        {
            let first = input
                .servers
                .iter()
                .find(|x| x.identity.is_named("2dirs+proxy"))
                .unwrap();
            let expected = DirsProxy(
                vec![
                    DirRoute {
                        dir: "another".to_string(),
                    },
                    DirRoute {
                        dir: "another_2".to_string(),
                    },
                ],
                ProxyRoute {
                    proxy: "example.com".to_string(),
                },
            );

            let actual = first.routes.iter().fold(HandlerStack::None, |s, route| {
                routes_to_stack(s, route.clone())
            });

            assert_eq!(actual, expected);
        }

        {
            let first = input
                .servers
                .iter()
                .find(|s| s.identity.is_named("2dirs"))
                .unwrap();
            let expected = Dirs(vec![
                DirRoute {
                    dir: "public".to_string(),
                },
                DirRoute {
                    dir: ".".to_string(),
                },
            ]);

            let actual = first.routes.iter().fold(HandlerStack::None, |s, route| {
                routes_to_stack(s, route.clone())
            });

            assert_eq!(actual, expected);
        }

        Ok(())
    }
}
