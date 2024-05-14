use bsnext_input::server_config::Identity;
use bsnext_input::{md, Input};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct MdExample;

impl MdExample {
    pub fn into_input(self, identity: Option<Identity>) -> Input {
        let input_str = include_str!("../../../examples/md-single/md-single.md");
        let mut input = md::md_to_input(input_str).expect("example cannot fail?");
        let server = input
            .servers
            .first_mut()
            .expect("example must have 1 server");
        server.identity = identity.unwrap_or_else(Identity::named);
        input
    }
}
