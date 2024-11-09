use bsnext_input::path_def::PathDef;
use bsnext_input::route::{Route, RouteKind};
use bsnext_input::server_config::ServerIdentity;
use bsnext_md::md_to_input;
use std::str::FromStr;

#[test]
fn test_single() -> anyhow::Result<()> {
    // let input = include_str!("../../examples/md-single/md-single.md");
    let input = r#"

# Demo 2

```yaml bslive_route
path: /app.css
```

```css
body {
    background: blue
}
```
        "#;
    let config = md_to_input(&input).expect("unwrap");
    let server_1 = config.servers.first().unwrap();
    assert_eq!(
        server_1.routes[0],
        Route {
            path: "/app.css".parse()?,
            kind: RouteKind::new_raw("body {\n    background: blue\n}"),
            ..Default::default()
        }
    );
    Ok(())
}

#[test]
fn test_2_consecutive() -> anyhow::Result<()> {
    // let input = include_str!("../../examples/md-single/md-single.md");
    let input = r#"

```yaml bslive_route
path: /app.css
```

```css
body {
    background: blue
}
```

Some other text

```yaml bslive_route
path: /app2.css
```

```css
body {
    background: blue
}
```
        "#;
    let config = md_to_input(&input).expect("unwrap");
    let server_1 = config.servers.first().unwrap();
    assert_eq!(
        server_1.routes[0],
        Route {
            path: PathDef::from_str("/app.css")?,
            kind: RouteKind::new_raw("body {\n    background: blue\n}"),
            ..Default::default()
        }
    );
    assert_eq!(
        server_1.routes[1],
        Route {
            path: PathDef::from_str("/app2.css")?,
            kind: RouteKind::new_raw("body {\n    background: blue\n}"),
            ..Default::default()
        }
    );
    Ok(())
}

#[test]
fn test_parse_with_elements_in_gaps() -> anyhow::Result<()> {
    let markdown = r#"
# Before

```yaml bslive_input
servers:
  - bind_address: 0.0.0.0:3001
    routes:
        - path: /health
          raw: OK
```

```yaml bslive_route
path: /
```

in between?

```html
<p>hello world</p>
```

```yaml bslive_route
path: /abc
```
```html
<p>hello world 2</p>
```

# Before
        "#;
    let input = md_to_input(&markdown).expect("unwrap");
    let server_1 = input.servers.first().unwrap();
    let expected_id = ServerIdentity::Address {
        bind_address: "0.0.0.0:3001".into(),
    };
    assert_eq!(server_1.identity, expected_id);
    assert_eq!(server_1.routes.len(), 3);
    assert_eq!(
        server_1.routes[0],
        Route {
            path: PathDef::from_str("/health")?,
            kind: RouteKind::new_raw("OK"),
            ..Default::default()
        }
    );
    assert_eq!(
        server_1.routes[1],
        Route {
            path: PathDef::from_str("/")?,
            kind: RouteKind::new_html("<p>hello world</p>"),
            ..Default::default()
        }
    );
    assert_eq!(
        server_1.routes[2],
        Route {
            path: PathDef::from_str("/abc")?,
            kind: RouteKind::new_html("<p>hello world 2</p>"),
            ..Default::default()
        }
    );
    Ok(())
}

fn default_md_assertions(input: &str) -> anyhow::Result<()> {
    let input = md_to_input(&input).expect("unwrap");
    let server_1 = input.servers.first().unwrap();
    let expected_id = ServerIdentity::Address {
        bind_address: "0.0.0.0:5001".into(),
    };
    assert_eq!(server_1.identity, expected_id);
    assert_eq!(server_1.routes.len(), 2);
    let paths = server_1
        .routes
        .iter()
        .map(|r| r.path.as_str())
        .collect::<Vec<&str>>();

    assert_eq!(paths, vec!["/app.css", "/"]);
    Ok(())
}

#[test]
fn test_from_example_str() -> anyhow::Result<()> {
    let input_str = include_str!("../../../examples/md-single/md-single.md");
    default_md_assertions(input_str)
}

#[test]
fn test_from_example_str_frontmatter() -> anyhow::Result<()> {
    let input_str = include_str!("../../../examples/md-single/frontmatter.md");
    default_md_assertions(input_str)
}
