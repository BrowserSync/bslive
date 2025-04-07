use bsnext_html::HtmlFs;
use bsnext_input::route::RouteKind;
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::startup::StartupContext;
use bsnext_input::{InputArgs, InputCreation, InputCtx};
use insta::assert_debug_snapshot;
use std::path::PathBuf;

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
    let startup_ctx = StartupContext::default();
    let ctx = InputCtx::new(&idens, None, &startup_ctx, None);
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
    let startup_ctx = StartupContext::default();
    let ctx = InputCtx::new(&idens, None, &startup_ctx, None);
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
    let startup_ctx = StartupContext::default();
    let ctx = InputCtx::new(&[ident.clone()], None, &startup_ctx, None);
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
    let startup_ctx = StartupContext::default();
    let ctx = InputCtx::new(&[], Some(input_args), &startup_ctx, None);
    let as_input = HtmlFs::from_input_str(INPUT, &ctx)?;
    let first = as_input.servers.get(0).unwrap();
    assert_eq!(first.identity, ident);
    Ok(())
}

const INPUT_WITH_META: &str = r#"
<main>
    <h1>Test!</h1>
    <abc-shane></abc-shane>
</main>"#;

#[test]
fn test_html_playground_with_serve() -> anyhow::Result<()> {
    let ident = ServerIdentity::Address {
        bind_address: String::from("0.0.0.0:8080"),
    };
    let input_args = InputArgs { port: Some(8080) };
    let startup_ctx = StartupContext::new("/users/shane");
    let file_path = Some(PathBuf::from("/users/shane/bslive.yml"));
    let ctx = InputCtx::new(&[], Some(input_args), &startup_ctx, file_path.as_ref());
    let as_input = HtmlFs::from_input_str(INPUT_WITH_META, &ctx)?;
    let first = as_input.servers.get(0).unwrap();
    assert_eq!(first.identity, ident);
    let routes = first.combined_routes();
    let found = routes
        .iter()
        .find(|x| matches!(x.kind, RouteKind::Dir(..)))
        .expect("must find dir");
    assert_debug_snapshot!(found.path);
    assert_debug_snapshot!(found.kind);
    Ok(())
}

const INPUT_WITH_META_DEFAULT: &str = r#"
<meta name="bslive serve-dir" />
<main>
    <h1>Test!</h1>
    <abc-shane></abc-shane>
</main>"#;

/// In this test, I am making sure `bslive serve-dir` works the same as the long-hand version above
#[test]
fn test_html_playground_with_meta_default() -> anyhow::Result<()> {
    let ident = ServerIdentity::Address {
        bind_address: String::from("0.0.0.0:8080"),
    };
    let input_args = InputArgs { port: Some(8080) };
    let startup_ctx = StartupContext::default();
    let ctx = InputCtx::new(&[], Some(input_args), &startup_ctx, None);
    let as_input = HtmlFs::from_input_str(INPUT_WITH_META_DEFAULT, &ctx)?;
    let first = as_input.servers.get(0).unwrap();
    assert_eq!(first.identity, ident);
    let routes = first.combined_routes();
    let found2 = routes
        .iter()
        .find(|x| matches!(x.kind, RouteKind::Dir(..)))
        .expect("must find dir");
    assert_debug_snapshot!(found2.path);
    assert_debug_snapshot!(found2.kind);
    Ok(())
}
