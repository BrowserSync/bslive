use bsnext_input::{Input, InputWriter};

pub struct HtmlWriter;

impl InputWriter for HtmlWriter {
    fn input_to_str(&self, input: &Input) -> String {
        if input.servers.is_empty() {
            todo!("html requires at least 1 server definition")
        }
        if input.servers.len() > 1 {
            todo!("more than 1 server not supported yet")
        }
        let server = input.servers.first().expect("must access first");
        let Some(playground) = &server.playground else {
            todo!("only playground is supported in HTML for now")
        };
        let mut blocks: Vec<String> = vec![];
        if let Some(css) = &playground.css {
            blocks.push("<style>".into());
            let indented = indent::indent_all_by(4, css);
            blocks.push(indented);
            blocks.push("</style>".into());
        }
        blocks.push(playground.html.clone());
        if let Some(js) = &playground.js {
            blocks.push("<script type=\"module\">".into());
            let indented = indent::indent_all_by(4, js);
            blocks.push(indented);
            blocks.push("</script>".into());
        }
        blocks.join("\n")
    }
}

#[test]
fn test_html_writer_for_playground() {
    use bsnext_input::playground::Playground;
    use bsnext_input::server_config::ServerConfig;
    let css = r#"body {
    background: red;
}"#;
    let js = r#"console.log("hello world!")"#;
    let playground = Playground {
        html: "<p>Hello world</p>".to_string(),
        js: Some(js.to_string()),
        css: Some(css.to_string()),
    };
    let mut input = Input::default();
    let mut server = ServerConfig::default();
    server.playground = Some(playground);
    input.servers.push(server);
    let output = HtmlWriter.input_to_str(&input);
    insta::assert_snapshot!(output);
}
