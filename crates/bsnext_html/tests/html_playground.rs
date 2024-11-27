use bsnext_html::HtmlFs;
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::{InputArgs, InputCreation, InputCtx};
use insta::assert_debug_snapshot;

const INPUT: &str = r#"
<style>
    p {
        color: red;
    }

    abc-shane {
        color: red;
    }
</style>
<main>
    <h1>Test!</h1>
    <abc-shane></abc-shane>
</main>
<script type="module">
    class BSlive extends HTMLElement {
        connectedCallback() {
            this.innerHTML = "Hello world";
        }
    }

    customElements.define("abc-shane", BSlive);
</script>"#;

#[test]
fn test_html_playground_content() -> anyhow::Result<()> {
    let idens = vec![];
    let ctx = InputCtx::new(&idens, None);
    let as_input = HtmlFs::from_input_str(INPUT, &ctx)?;
    let Some(server) = as_input.servers.get(0) else {
        return Err(anyhow::anyhow!("no server"));
    };
    let routes = server.combined_routes();
    let html = routes.get(0).unwrap();
    let js = routes.get(1).unwrap();
    let css = routes.get(2).unwrap();
    assert_debug_snapshot!(html.kind);
    assert_debug_snapshot!(js.kind);
    assert_debug_snapshot!(css.kind);
    Ok(())
}
#[test]
fn test_html_playground_without_server_id() -> anyhow::Result<()> {
    let idens = vec![];
    let ctx = InputCtx::new(&idens, None);
    let as_input = HtmlFs::from_input_str(INPUT, &ctx)?;
    assert_eq!(as_input.servers.len(), 1);
    let first = as_input.servers.get(0).unwrap();
    let is_named = matches!(first.identity, ServerIdentity::Named { .. });
    assert_eq!(is_named, true);
    Ok(())
}
#[test]
fn test_html_playground_with_server_id() -> anyhow::Result<()> {
    let ident = ServerIdentity::Address {
        bind_address: String::from("127.0.0.1:8080"),
    };
    let ctx = InputCtx::new(&[ident.clone()], None);
    let as_input = HtmlFs::from_input_str(INPUT, &ctx)?;

    assert_eq!(as_input.servers.len(), 1);
    let first = as_input.servers.get(0).unwrap();
    assert_eq!(ident, first.identity);
    Ok(())
}
#[test]
fn test_html_playground_with_port() -> anyhow::Result<()> {
    let ident = ServerIdentity::Address {
        bind_address: String::from("0.0.0.0:8080"),
    };
    let input_args = InputArgs { port: Some(8080) };
    let ctx = InputCtx::new(&[], Some(input_args));
    let as_input = HtmlFs::from_input_str(INPUT, &ctx)?;
    let first = as_input.servers.get(0).unwrap();
    assert_eq!(first.identity, ident);
    Ok(())
}
