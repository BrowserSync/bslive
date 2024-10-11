use bsnext_input::route::{Route, RouteKind};
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::{
    server_config::{self},
    Input,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BasicExample;

impl BasicExample {
    pub fn into_input(self, identity: Option<ServerIdentity>) -> Input {
        let server = server_config::ServerConfig {
            identity: identity.unwrap_or_else(ServerIdentity::named),
            routes: vec![
                Route {
                    path: "/".to_string().parse().unwrap(),
                    kind: RouteKind::new_html(include_str!(
                        "../../../examples/basic/public/index.html"
                    )),
                    ..Default::default()
                },
                Route {
                    path: "/styles.css".to_string().parse().unwrap(),
                    kind: RouteKind::new_raw(include_str!(
                        "../../../examples/basic/public/styles.css"
                    )),
                    ..Default::default()
                },
                Route {
                    path: "/script.js".to_string().parse().unwrap(),
                    kind: RouteKind::new_raw(include_str!(
                        "../../../examples/basic/public/script.js"
                    )),
                    ..Default::default()
                },
                Route {
                    path: "/reset.css".to_string().parse().unwrap(),
                    kind: RouteKind::new_raw(include_str!(
                        "../../../examples/basic/public/reset.css"
                    )),
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
