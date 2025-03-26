use bsnext_input::playground::Playground;
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use bsnext_input::{Input, InputCreation, InputCtx, InputError};
use std::fs::read_to_string;
use std::path::Path;

pub struct JsFs;

const HTML: &str = r#"
<h1>BSLive JS Playground</h1>
<main>Main</main>
"#;

impl InputCreation for JsFs {
    fn from_input_path<P: AsRef<Path>>(path: P, ctx: &InputCtx) -> Result<Input, Box<InputError>> {
        let str = read_to_string(path).map_err(|e| Box::new(e.into()))?;

        // start an empty playground
        let playground = Playground {
            html: HTML.to_string(),
            css: None,
            js: Some(str),
        };

        // Now start to build up the input
        let mut input = Input::default();

        // 1: first try prev
        // 2: next try if 'port' was provided
        // 3: finally, make one up
        let iden = ctx
            .first_id()
            .or_else(|| ServerIdentity::from_port_or_named(ctx.port()).ok())
            .unwrap_or_default();

        // Create the server
        let server = ServerConfig {
            identity: iden,
            playground: Some(playground),
            // routes,
            ..Default::default()
        };

        // Add it to the input
        input.servers.push(server);

        Ok(input)
    }

    fn from_input_str<P: AsRef<str>>(
        _content: P,
        _ctx: &InputCtx,
    ) -> Result<Input, Box<InputError>> {
        todo!()
    }
}
