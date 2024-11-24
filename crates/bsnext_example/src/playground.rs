use bsnext_html::HtmlFs;
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::{InputCreation, InputCtx, InputSource, InputSourceKind};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct PlaygroundExample;

impl InputSource for PlaygroundExample {
    fn into_input(self, identity: Option<ServerIdentity>) -> InputSourceKind {
        let input_str = include_str!("../../../examples/html/playground.html");
        let mut input = HtmlFs::from_input_str(input_str, &InputCtx::default()).unwrap();

        // update the server identity if it was provided
        if let (Some(server), Some(identity)) = (input.servers.get_mut(0), identity) {
            server.identity = identity;
        }

        InputSourceKind::Type(input)
    }
}
