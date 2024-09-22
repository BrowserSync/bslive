use crate::route::{RawRoute, Route, RouteKind};
use crate::server_config::ServerConfig;
use crate::Input;

use crate::path_def::PathDef;
use markdown::mdast::Node;
use markdown::{Constructs, ParseOptions};
use mime_guess::get_mime_extensions_str;
use nom::branch::alt;
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::separated_pair;
use nom::{error::ParseError, IResult};
use serde_json::json;
use std::cmp::PartialEq;
use std::str::FromStr;

fn parser_for(k: BsLiveKinds) -> impl Fn(&[Node]) -> IResult<&[Node], &Node> {
    move |input: &[Node]| {
        if input.is_empty() || input[0].kind() != k {
            Err(nom::Err::Error(ParseError::from_error_kind(
                input,
                nom::error::ErrorKind::Tag,
            )))
        } else {
            Ok((&input[1..], &input[0]))
        }
    }
}

#[derive(Debug, PartialEq, PartialOrd)]
pub enum BsLiveKinds {
    Input,
    Route,
    Body,
    Ignored,
}

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum MarkdownError {
    #[error("could not parse Markdown: {0}")]
    ParseError(String),
    #[error("invalid markdown format: {0}")]
    InvalidFormat(String),
}

trait BsLive {
    fn kind(&self) -> BsLiveKinds;
    fn is_input(&self) -> bool;
    fn is_route(&self) -> bool;
    fn is_body(&self) -> bool;
    #[allow(dead_code)]
    fn raw_value(&self) -> Option<String>;
}

impl TryInto<Input> for &Node {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Input, Self::Error> {
        if !self.is_input() {
            return Err(anyhow::anyhow!("not an input type"));
        }
        match self {
            Node::Code(code) => {
                let config: Input = serde_yaml::from_str(&code.value)?;
                Ok(config)
            }
            Node::Yaml(yaml) => {
                let config: Input = serde_yaml::from_str(&yaml.value)?;
                Ok(config)
            }
            _ => Err(anyhow::anyhow!("unreachable")),
        }
    }
}

impl TryInto<Input> for Vec<Node> {
    type Error = MarkdownError;
    fn try_into(self) -> Result<Input, Self::Error> {
        nodes_to_input(&self)
    }
}

impl TryInto<Route> for (&Node, &Node) {
    type Error = anyhow::Error;

    fn try_into(self) -> Result<Route, Self::Error> {
        match (self.0.is_route(), self.1.is_body()) {
            (true, true) => match self.0 {
                Node::Code(code)
                    if code
                        .lang
                        .as_ref()
                        .is_some_and(|l| l == "yaml" || l == "yml") =>
                {
                    #[derive(serde::Deserialize)]
                    struct PathOnly {
                        path: String,
                    }
                    let r: PathOnly = serde_yaml::from_str(&code.value)?;
                    let route_kind = route_kind_from_body_node(self.1)?;
                    let route = Route {
                        path: PathDef::from_str(&r.path)?,
                        kind: route_kind,
                        ..Default::default()
                    };
                    Ok(route)
                }
                _ => Err(anyhow::anyhow!("unreachlable")),
            },
            _ => Err(anyhow::anyhow!("cannot create")),
        }
    }
}

fn route_kind_from_body_node(node: &Node) -> anyhow::Result<RouteKind> {
    match node {
        Node::Code(code) => {
            let value = code.value.clone();
            let rk = match code.lang.as_deref() {
                Some("html") => RouteKind::new_html(&value),
                Some("json") => RouteKind::new_json(serde_json::from_str(&value)?),
                Some("sse") => RouteKind::new_sse(value),
                Some(..) | None => RouteKind::new_raw(value),
            };
            Ok(rk)
        }
        _ => Err(anyhow::anyhow!("unsupported route kind")),
    }
}

impl BsLive for Node {
    fn kind(&self) -> BsLiveKinds {
        if self.is_input() {
            return BsLiveKinds::Input;
        }
        if self.is_route() {
            return BsLiveKinds::Route;
        }
        if self.is_body() {
            return BsLiveKinds::Body;
        }
        BsLiveKinds::Ignored
    }
    fn is_input(&self) -> bool {
        match self {
            // this is from front matter
            Node::Yaml(_yaml) => true,
            // code block with annotations
            Node::Code(code) => code
                .meta
                .as_ref()
                .is_some_and(|v| v.contains("bslive_input")),
            _ => false,
        }
    }

    fn is_route(&self) -> bool {
        match self {
            Node::Code(code) => code
                .meta
                .as_ref()
                .is_some_and(|v| v.contains("bslive_route")),
            _ => false,
        }
    }

    fn is_body(&self) -> bool {
        if self.is_input() || self.is_route() {
            return false;
        }
        matches!(self, Node::Code(..))
    }

    fn raw_value(&self) -> Option<String> {
        if self.is_body() {
            let Node::Code(code) = self else {
                unreachable!("shouldnt get here");
            };
            Some(code.value.clone())
        } else {
            None
        }
    }
}

enum Convert {
    None,
    Input(Input),
    Route(Route),
}

pub fn nodes_to_input(nodes: &[Node]) -> Result<Input, MarkdownError> {
    let mut routes = vec![];
    let mut server_config: Option<Input> = None;
    let mut parser = many0(alt((
        map(parser_for(BsLiveKinds::Ignored), |_v| Convert::None),
        map(parser_for(BsLiveKinds::Input), |v: &Node| {
            let as_config: Result<Input, _> = v.try_into();
            match as_config {
                Ok(config) => Convert::Input(config),
                Err(e) => unreachable!("? creating server config {:?}", e),
            }
        }),
        map(
            separated_pair(
                parser_for(BsLiveKinds::Route),
                many0(parser_for(BsLiveKinds::Ignored)),
                parser_for(BsLiveKinds::Body),
            ),
            |pair| {
                let as_route: Result<Route, _> = pair.try_into();
                match as_route {
                    Ok(route) => Convert::Route(route),
                    Err(e) => unreachable!("? {:?}", e),
                }
            },
        ),
    )));

    let results = parser(nodes);

    match results {
        Ok((_rest, matched)) => {
            for item in matched {
                match item {
                    Convert::None => {}
                    Convert::Input(input_from_md) => {
                        // todo: handle server config
                        if server_config.is_none() {
                            server_config = Some(input_from_md)
                        } else {
                            unreachable!("todo: support multiple 'input' blocks")
                        }
                    }
                    Convert::Route(route) => {
                        routes.push(route);
                    }
                }
            }
        }
        Err(e) => return Err(MarkdownError::InvalidFormat(e.to_string())),
    }

    match server_config.take() {
        // config was not set, use default
        None => {
            let mut input = Input::default();
            let server = ServerConfig {
                routes,
                ..Default::default()
            };
            input.servers.push(server);
            Ok(input)
        }
        // got some server config, use it.
        Some(mut input) => {
            if let Some(server) = input.servers.first_mut() {
                server.routes.extend(routes)
            }
            Ok(input)
        }
    }
}

fn str_to_nodes(input: &str) -> Result<Vec<Node>, MarkdownError> {
    let opts = ParseOptions {
        constructs: Constructs {
            frontmatter: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let root =
        markdown::to_mdast(input, &opts).map_err(|e| MarkdownError::ParseError(e.to_string()))?;
    match root {
        Node::Root(root) => Ok(root.children),
        _ => {
            unreachable!("?");
        }
    }
}

pub fn md_to_input(input: &str) -> Result<Input, MarkdownError> {
    let root = str_to_nodes(input)?;
    nodes_to_input(&root)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::server_config::ServerIdentity;

    #[test]
    fn test_single() -> anyhow::Result<()> {
        // let input = include_str!("../../examples/md-single/md-single.md");
        let input = r#"

# Demo 2

```yaml bslive_route
path: /app.css
```

```css
body {
    background: blue
}
```
        "#;
        let config = md_to_input(&input).expect("unwrap");
        let server_1 = config.servers.first().unwrap();
        assert_eq!(
            server_1.routes[0],
            Route {
                path: "/app.css".parse()?,
                kind: RouteKind::new_raw("body {\n    background: blue\n}"),
                ..Default::default()
            }
        );
        Ok(())
    }

    #[test]
    fn test_2_consecutive() -> anyhow::Result<()> {
        // let input = include_str!("../../examples/md-single/md-single.md");
        let input = r#"

```yaml bslive_route
path: /app.css
```

```css
body {
    background: blue
}
```

Some other text

```yaml bslive_route
path: /app2.css
```

```css
body {
    background: blue
}
```
        "#;
        let config = md_to_input(&input).expect("unwrap");
        let server_1 = config.servers.first().unwrap();
        assert_eq!(
            server_1.routes[0],
            Route {
                path: PathDef::from_str("/app.css")?,
                kind: RouteKind::new_raw("body {\n    background: blue\n}"),
                ..Default::default()
            }
        );
        assert_eq!(
            server_1.routes[1],
            Route {
                path: PathDef::from_str("/app2.css")?,
                kind: RouteKind::new_raw("body {\n    background: blue\n}"),
                ..Default::default()
            }
        );
        Ok(())
    }

    #[test]
    fn test_parse_with_elements_in_gaps() -> anyhow::Result<()> {
        let input = r#"
# Before

```yaml bslive_input
servers:
  - bind_address: 0.0.0.0:3001
    routes:
        - path: /health
          raw: OK
```

```yaml bslive_route
path: /
```

in between?

```html
<p>hello world</p>
```

```yaml bslive_route
path: /abc
```
```html
<p>hello world 2</p>
```

# Before
        "#;
        let config = md_to_input(&input).expect("unwrap");
        let server_1 = config.servers.first().unwrap();
        let expected_id = ServerIdentity::Address {
            bind_address: "0.0.0.0:3001".into(),
        };
        assert_eq!(server_1.identity, expected_id);
        assert_eq!(server_1.routes.len(), 3);
        assert_eq!(
            server_1.routes[0],
            Route {
                path: PathDef::from_str("/health")?,
                kind: RouteKind::new_raw("OK"),
                ..Default::default()
            }
        );
        assert_eq!(
            server_1.routes[1],
            Route {
                path: PathDef::from_str("/")?,
                kind: RouteKind::new_html("<p>hello world</p>"),
                ..Default::default()
            }
        );
        assert_eq!(
            server_1.routes[2],
            Route {
                path: PathDef::from_str("/abc")?,
                kind: RouteKind::new_html("<p>hello world 2</p>"),
                ..Default::default()
            }
        );
        Ok(())
    }

    fn default_md_assertions(input: &str) -> anyhow::Result<()> {
        let input = md_to_input(&input).expect("unwrap");
        let server_1 = input.servers.first().unwrap();
        let expected_id = ServerIdentity::Address {
            bind_address: "0.0.0.0:5001".into(),
        };
        assert_eq!(server_1.identity, expected_id);
        assert_eq!(server_1.routes.len(), 2);
        let paths = server_1
            .routes
            .iter()
            .map(|r| r.path.as_str())
            .collect::<Vec<&str>>();

        assert_eq!(paths, vec!["/app.css", "/"]);
        Ok(())
    }

    #[test]
    fn test_from_example_str() -> anyhow::Result<()> {
        let input_str = include_str!("../../../examples/md-single/md-single.md");
        default_md_assertions(input_str)
    }

    #[test]
    fn test_from_example_str_frontmatter() -> anyhow::Result<()> {
        let input_str = include_str!("../../../examples/md-single/frontmatter.md");
        default_md_assertions(input_str)
    }
}

pub fn input_to_str(input: &Input) -> String {
    let mut chunks = vec![];
    if let Some(server_config) = input.servers.first() {
        let without_routes = Input {
            servers: vec![ServerConfig {
                identity: server_config.identity.clone(),
                routes: vec![],
                ..Default::default()
            }],
        };
        let yml = serde_yaml::to_string(&without_routes).expect("never fail here");

        chunks.push(fenced_input(&yml));

        for route in &server_config.routes {
            let path_only = json!({"path": route.path.as_str()});
            let route_yaml = serde_yaml::to_string(&path_only).expect("never fail here on route?");
            chunks.push(fenced_route(&route_yaml));
            chunks.push(route_to_markdown(&route.kind, route.path.as_str()));
        }
    }
    for _x in input.servers.iter().skip(1) {
        todo!("not supported yet!")
    }
    chunks.join("\n")
}

fn route_to_markdown(kind: &RouteKind, path: &str) -> String {
    match kind {
        RouteKind::Raw(raw) => match raw {
            RawRoute::Html { html } => fenced_body("html", html),
            RawRoute::Json { .. } => todo!("unsupported json"),
            RawRoute::Raw { raw } => {
                let mime = mime_guess::from_path(path);
                let as_str = mime.first_or_text_plain();
                let as_str = get_mime_extensions_str(as_str.as_ref());
                if let Some(v) = as_str.and_then(|x| x.first()) {
                    fenced_body(v, raw)
                } else {
                    fenced_body("", raw)
                }
            }
            RawRoute::Sse { .. } => todo!("unsupported"),
        },
        RouteKind::Proxy(_) => todo!("unsupported"),
        RouteKind::Dir(_) => todo!("unsupported"),
    }
}

fn fenced_input(code: &str) -> String {
    format!("```yaml bslive_input\n{}```", code)
}

fn fenced_route(code: &str) -> String {
    format!("```yaml bslive_route\n{}```", code)
}

fn fenced_body(lang: &str, code: &str) -> String {
    format!("```{lang}\n{code}\n```")
}

#[cfg(test)]
mod test_serialize {
    use super::*;
    #[test]
    fn test_input_to_str() -> anyhow::Result<()> {
        let input_str = include_str!("../../../examples/md-single/md-single.md");
        let input = md_to_input(&input_str).expect("unwrap");
        let _output = input_to_str(&input);
        let input = md_to_input(&input_str).expect("unwrapped 2");
        assert_eq!(input.servers.len(), 1);
        assert_eq!(input.servers.first().unwrap().routes.len(), 2);
        Ok(())
    }
}
