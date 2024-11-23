use bsnext_input::playground::Playground;
use bsnext_input::server_config::{ServerConfig, ServerIdentity};
use bsnext_input::{Input, InputCreation, InputError};
use std::fs::read_to_string;
use std::path::Path;

pub struct HtmlFs;

impl InputCreation for HtmlFs {
    fn from_input_path<P: AsRef<Path>>(
        path: P,
        server_ids: Vec<ServerIdentity>,
    ) -> Result<Input, Box<InputError>> {
        let str = read_to_string(path).map_err(|e| Box::new(e.into()))?;
        let input = playground_html_str_to_input(&str, server_ids)
            .map_err(|e| Box::new(InputError::HtmlError(e.to_string())))?;
        Ok(input)
    }

    fn from_input_str<P: AsRef<str>>(content: P) -> Result<Input, Box<InputError>> {
        let input = playground_html_str_to_input(&content.as_ref(), vec![])
            .map_err(|e| Box::new(InputError::HtmlError(e.to_string())))?;
        Ok(input)
    }
}

fn playground_html_str_to_input(
    html: &str,
    server_ids: Vec<ServerIdentity>,
) -> Result<Input, Box<InputError>> {
    use unindent::unindent;
    let mut document = scraper::Html::parse_fragment(html);
    let style = scraper::Selector::parse("style:first-of-type").unwrap();
    let script = scraper::Selector::parse("script:first-of-type").unwrap();
    let mut style_elems = document.select(&style);
    let mut script_elems = document.select(&script);
    let mut removals = vec![];

    let mut playground = Playground {
        html: "".to_string(),
        css: None,
        js: None,
    };

    if let Some(style) = style_elems.next() {
        removals.push(style.id());
        let t = style.text().nth(0).unwrap();
        let unindented = unindent(t);
        playground.css = Some(unindented);
    }

    if let Some(script) = script_elems.next() {
        removals.push(script.id());
        let t = script.text().nth(0).unwrap();
        let unindented = unindent(t);
        playground.js = Some(unindented);
    }

    for x in removals {
        document.tree.get_mut(x).unwrap().detach();
    }

    let as_html = document.html();
    let trimmed = as_html
        .strip_prefix("<html>")
        .unwrap()
        .strip_suffix("</html>")
        .unwrap();
    let un_indented = unindent(trimmed);
    playground.html = un_indented;
    let mut input = Input::default();
    let server = ServerConfig {
        routes: playground.as_routes(),
        identity: server_ids
            .get(0)
            .map(ToOwned::to_owned)
            .unwrap_or_else(|| ServerIdentity::named()),
        ..Default::default()
    };
    input.servers.push(server);
    // dbg!(&input.servers.get(0).unwrap().identity);
    Ok(input)
}

#[test]
fn has_selector() {
    let mut html = r#"
        huh?
        <style>
            @import url("reset.css")
            body {
                color: red;
            }
        </style>
        <main>
            <ul>
                <li>first</li>
                <li>second</li>
                <li>third</li>
            </ul>
        </main>
        <footer>
            <p><a href=""></a></p>
        </footer>
        <script type="module">
            console.log(document.querySelectorAll("li"))
        </script>"#;

    let mut document = scraper::Html::parse_fragment(html);

    #[derive(Debug)]
    struct Item {
        pub html: String,
        pub css: Option<String>,
        pub js: Option<String>,
    }

    let mut item = Item {
        html: "".to_string(),
        css: None,
        js: None,
    };

    dbg!(&item);

    // fs::write("origin.html", item.html).unwrap();
    // fs::write("origin.js", item.js.unwrap()).unwrap();
    // fs::write("origin.css", item.css.unwrap()).unwrap();
}
