use bsnext_input::route::{Route, RouteKind};
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::{
    server_config::{self},
    Input,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct LitExample;

impl LitExample {
    pub fn into_input(self, identity: Option<ServerIdentity>) -> Input {
        let server = server_config::ServerConfig {
            identity: identity.unwrap_or_else(ServerIdentity::named),
            routes: vec![
                Route {
                    path: "/".to_string(),
                    kind: RouteKind::Html {
                        html: include_str!("../../../examples/lit/index.html").to_owned(),
                    },
                    ..Default::default()
                },
                Route {
                    path: "/lit.js".to_string(),
                    kind: RouteKind::Raw {
                        raw: include_str!("../../../examples/lit/lit.js").to_owned(),
                    },
                    ..Default::default()
                },
            ],
            watchers: vec![],
        };
        Input {
            servers: vec![server],
        }
    }
}
