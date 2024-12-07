use bsnext_input::playground::Playground;
use bsnext_input::route::Route;
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use bsnext_input::{Input, InputCreation, InputCtx, InputError};
use std::fs::read_to_string;
use std::path::Path;

pub mod html_writer;

pub struct HtmlFs;

impl InputCreation for HtmlFs {
    fn from_input_path<P: AsRef<Path>>(path: P, ctx: &InputCtx) -> Result<Input, Box<InputError>> {
        let str = read_to_string(path).map_err(|e| Box::new(e.into()))?;
        let input = playground_html_str_to_input(&str, ctx)
            .map_err(|e| Box::new(InputError::HtmlError(e.to_string())))?;
        Ok(input)
    }

    fn from_input_str<P: AsRef<str>>(content: P, ctx: &InputCtx) -> Result<Input, Box<InputError>> {
        let input = playground_html_str_to_input(content.as_ref(), ctx)
            .map_err(|e| Box::new(InputError::HtmlError(e.to_string())))?;
        Ok(input)
    }
}

fn playground_html_str_to_input(html: &str, ctx: &InputCtx) -> Result<Input, Box<InputError>> {
    use unindent::unindent;

    // parse the HTML
    let mut document = scraper::Html::parse_fragment(html);

    let style = scraper::Selector::parse("style:first-of-type").unwrap();
    let script = scraper::Selector::parse("script:first-of-type").unwrap();
    let meta = scraper::Selector::parse("meta[name][content]").unwrap();

    let mut style_elems = document.select(&style);
    let mut script_elems = document.select(&script);
    let _meta_elems = document.select(&meta);
    let mut node_ids_to_remove = vec![];

    // start an empty playground
    let mut playground = Playground {
        html: "".to_string(),
        css: None,
        js: None,
    };

    let mut routes = vec![];

    if let Some(style) = style_elems.next() {
        node_ids_to_remove.push(style.id());
        let t = style.text().next().unwrap();
        let unindented = unindent(t);
        playground.css = Some(unindented);
    }

    if let Some(script) = script_elems.next() {
        node_ids_to_remove.push(script.id());
        let t = script.text().next().unwrap();
        let unindented = unindent(t);
        playground.js = Some(unindented);
    }

    for x in _meta_elems {
        let name = x.attr("name");
        let content = x.attr("content");
        match (name, content) {
            (Some(name), Some(content)) => {
                let joined = format!("{} {}", name, content);
                let r = Route::from_cli_str(joined);
                if let Ok(route) = r {
                    routes.push(route);
                    node_ids_to_remove.push(x.id());
                }
            }
            _ => todo!("not supported!"),
        }
    }

    for node_id in node_ids_to_remove {
        document.tree.get_mut(node_id).unwrap().detach();
    }

    // grab the HTML
    let as_html = document.html();
    let trimmed = as_html
        .strip_prefix("<html>")
        .unwrap()
        .strip_suffix("</html>")
        .unwrap();
    let un_indented = unindent(trimmed);
    playground.html = un_indented;

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
        routes,
        ..Default::default()
    };

    // Add it to the input
    input.servers.push(server);

    Ok(input)
}
