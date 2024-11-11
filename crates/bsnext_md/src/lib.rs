pub mod md_fs;
pub mod md_writer;

use bsnext_input::path_def::PathDef;
use bsnext_input::playground::Playground;
use bsnext_input::route::{Route, RouteKind};
use bsnext_input::server_config::ServerConfig;
use bsnext_input::Input;
use markdown::mdast::Node;
use markdown::{Constructs, ParseOptions};
use nom::branch::alt;
use nom::combinator::map;
use nom::multi::many0;
use nom::sequence::{pair, separated_pair};
use nom::{error::ParseError, IResult};
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
    PlaygroundHtml,
    PlaygroundCSS,
    PlaygroundJS,
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

fn node_to_input(node: &Node) -> Result<Input, anyhow::Error> {
    if !node.is_input() {
        return Err(anyhow::anyhow!("not an input type"));
    }
    match node {
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

fn pair_to_route((first, second): (&Node, &Node)) -> Result<Route, anyhow::Error> {
    match (first.is_route(), second.is_body()) {
        (true, true) => match first {
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
                let route_kind = route_kind_from_body_node(second)?;
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

trait BsLive {
    fn kind(&self) -> BsLiveKinds;
    fn is_input(&self) -> bool;
    fn is_route(&self) -> bool;
    fn is_body(&self) -> bool;
    fn is_playground_html(&self) -> bool;
    fn is_playground_css(&self) -> bool;
    fn is_playground_js(&self) -> bool;
    #[allow(dead_code)]
    fn raw_value(&self) -> Option<String>;
}

impl BsLive for Node {
    fn kind(&self) -> BsLiveKinds {
        if self.is_playground_html() {
            return BsLiveKinds::PlaygroundHtml;
        }
        if self.is_input() {
            return BsLiveKinds::Input;
        }
        if self.is_route() {
            return BsLiveKinds::Route;
        }
        if self.is_playground_css() {
            return BsLiveKinds::PlaygroundCSS;
        }
        if self.is_playground_js() {
            return BsLiveKinds::PlaygroundJS;
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

    fn is_playground_html(&self) -> bool {
        match self {
            Node::Code(code) => {
                code.lang.as_ref().is_some_and(|v| v == "html")
                    && code.meta.as_ref().is_some_and(|v| v.contains("playground"))
            }
            _ => false,
        }
    }

    fn is_playground_css(&self) -> bool {
        match self {
            Node::Code(code) => code.lang.as_ref().is_some_and(|v| v == "css"),
            _ => false,
        }
    }

    fn is_playground_js(&self) -> bool {
        match self {
            Node::Code(code) => code
                .lang
                .as_ref()
                .is_some_and(|v| v == "js" || v == "javascript"),
            _ => false,
        }
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
    Playground(Playground),
}

pub fn nodes_to_input(nodes: &[Node]) -> Result<Input, MarkdownError> {
    let mut routes = vec![];
    let mut input: Option<Input> = None;
    let mut parser = many0(alt((
        map(parser_for(BsLiveKinds::Ignored), |_v| Convert::None),
        map(parser_for(BsLiveKinds::Input), |v: &Node| {
            let as_config: Result<Input, _> = node_to_input(v);
            match as_config {
                Ok(config) => Convert::Input(config),
                Err(e) => unreachable!("? creating server config {:?}", e),
            }
        }),
        map(
            separated_pair(
                parser_for(BsLiveKinds::Route),
                many0(parser_for(BsLiveKinds::Ignored)),
                alt((
                    parser_for(BsLiveKinds::Body),
                    parser_for(BsLiveKinds::PlaygroundCSS),
                    parser_for(BsLiveKinds::PlaygroundJS),
                )),
            ),
            |route_body_pair| {
                let as_route: Result<Route, _> = pair_to_route(route_body_pair);
                match as_route {
                    Ok(route) => Convert::Route(route),
                    Err(_e) => {
                        // todo: add error handling here?
                        Convert::None
                    }
                }
            },
        ),
        map(
            pair(
                parser_for(BsLiveKinds::PlaygroundHtml),
                many0(alt((
                    parser_for(BsLiveKinds::PlaygroundCSS),
                    parser_for(BsLiveKinds::PlaygroundJS),
                ))),
            ),
            |(a, nodes): (&Node, Vec<&Node>)| {
                let mut pl = Playground {
                    html: code_val(a).to_string(),
                    js: None,
                    css: None,
                };

                for node in nodes {
                    if let Node::Code(code) = &node {
                        match code.lang.as_ref() {
                            None => {}
                            Some(lang) if lang == "js" => {
                                if let Some(js) = pl.js.as_mut() {
                                    *js = code.value.clone();
                                } else {
                                    pl.js = Some(code.value.clone());
                                }
                            }
                            Some(lang) if lang == "css" => {
                                if let Some(css) = pl.css.as_mut() {
                                    *css = code.value.clone();
                                } else {
                                    pl.css = Some(code.value.clone());
                                }
                            }
                            Some(_) => {
                                unreachable!("unsupposted");
                            }
                        }
                    }
                }

                Convert::Playground(pl)
            },
        ),
    )));

    let results = parser(nodes);
    let mut playground: Option<Playground> = None;

    match results {
        Ok((_rest, matched)) => {
            for item in matched {
                match item {
                    Convert::None => {}
                    Convert::Input(input_from_md) => {
                        // todo: handle server config
                        if input.is_none() {
                            input = Some(input_from_md)
                        } else {
                            unreachable!("todo: support multiple 'input' blocks")
                        }
                    }
                    Convert::Route(route) => {
                        routes.push(route);
                    }
                    Convert::Playground(pl) => {
                        if playground.is_none() {
                            playground = Some(pl)
                        }
                    }
                }
            }
        }
        Err(e) => return Err(MarkdownError::InvalidFormat(e.to_string())),
    }

    match input.take() {
        // config was not set, use default
        None => {
            let mut input = Input::default();
            let server = ServerConfig {
                routes,
                ..Default::default()
            };
            input.servers.push(server);
            if let Some(s) = input.servers.get_mut(0) {
                s.playground = playground
            }
            Ok(input)
        }
        // got some server config, use it.
        Some(mut input) => {
            if let Some(server) = input.servers.first_mut() {
                server.routes.extend(routes);
                server.playground = playground;
            }
            Ok(input)
        }
    }
}

fn code_val(n: &Node) -> &str {
    match n {
        Node::Code(code) => code.value.as_str(),
        _ => "",
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
