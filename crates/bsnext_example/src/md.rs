use bsnext_input::server_config::ServerIdentity;
use bsnext_input::{InputSource, InputSourceKind};
use bsnext_md::md_to_input;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MdExample;

impl InputSource for MdExample {
    fn into_input(&self, identity: Option<ServerIdentity>) -> InputSourceKind {
        let input_str = include_str!("../../../examples/markdown/single.md");
        let mut input = md_to_input(input_str).expect("example cannot fail?");
        let server = input
            .servers
            .first_mut()
            .expect("example must have 1 server");
        server.identity = identity.unwrap_or_else(ServerIdentity::named);
        InputSourceKind::Type(input)
    }
}
