use bsnext_input::route::{Route, RouteKind};
use bsnext_input::server_config::Identity;
use bsnext_input::{
    server_config::{self},
    Input,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BasicExample;

impl BasicExample {
    pub fn into_input(self, identity: Option<Identity>) -> Input {
        let server = server_config::ServerConfig {
            identity: identity.unwrap_or_else(Identity::named),
            routes: vec![
                Route {
                    path: "/".to_string(),
                    kind: RouteKind::Html {
                        html: include_str!("../../../examples/basic/public/index.html").to_owned(),
                    },
                    ..Default::default()
                },
                Route {
                    path: "/styles.css".to_string(),
                    kind: RouteKind::Raw {
                        raw: include_str!("../../../examples/basic/public/styles.css").to_owned(),
                    },
                    ..Default::default()
                },
                Route {
                    path: "/script.js".to_string(),
                    kind: RouteKind::Raw {
                        raw: include_str!("../../../examples/basic/public/script.js").to_owned(),
                    },
                    ..Default::default()
                },
                Route {
                    path: "/reset.css".to_string(),
                    kind: RouteKind::Raw {
                        raw: include_str!("../../../examples/basic/public/reset.css").to_owned(),
                    },
                    ..Default::default()
                },
            ],
            ..Default::default()
        };
        Input {
            servers: vec![server],
        }
    }
}
