use bsnext_input::route::{RawRoute, RouteKind};
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::{Input, InputWriter};
use mime_guess::get_mime_extensions_str;
use serde_json::json;

pub struct MdWriter;

impl InputWriter for MdWriter {
    fn input_to_str(&self, input: &Input) -> String {
        _input_to_str(input)
    }
}

fn _input_to_str(input: &Input) -> String {
    let mut chunks = vec![];
    if let Some(server_config) = input.servers.first() {
        #[derive(Debug, serde::Serialize)]
        struct FakeServerConfig {
            #[serde(flatten)]
            identity: ServerIdentity,
        }
        #[derive(Debug, serde::Serialize)]
        struct FakeInput {
            servers: Vec<FakeServerConfig>,
        }
        let just_identity = FakeInput {
            servers: vec![FakeServerConfig {
                identity: server_config.identity.clone(),
            }],
        };
        let yml = serde_yaml::to_string(&just_identity).expect("never fail here");

        chunks.push(fenced_input(&yml));

        if let Some(playground) = &server_config.playground {
            chunks.push(fenced_playground(&playground.html));
            if let Some(css) = &playground.css {
                chunks.push(fenced_body("css", css));
            }
            if let Some(js) = &playground.js {
                chunks.push(fenced_body("js", js));
            }
        }

        for route in server_config.routes() {
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

fn fenced_playground(code: &str) -> String {
    format!("```html playground\n{}\n```", code)
}

fn fenced_body(lang: &str, code: &str) -> String {
    format!("```{lang}\n{code}\n```")
}
