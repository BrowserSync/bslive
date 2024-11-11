use bsnext_md::md_to_input;

#[test]
fn test_md_playground() -> anyhow::Result<()> {
    // let input = include_str!("../../examples/markdown/markdown.md");
    let input = r#"

```html playground

<div class="main">
    Hello world!
</div>
```

```css
@import url("/reset.css");

:root {
    border: 50px solid pink;
    height: 100vh;
    overflow: hidden;
}
```

```js
console.log("hello world")
```

        "#;
    let config = md_to_input(&input).expect("unwrap");
    let first_server = config.servers.get(0).unwrap();
    let routes = first_server
        .playground
        .as_ref()
        .map(|x| x.as_routes())
        .unwrap();
    insta::assert_debug_snapshot!(routes);
    Ok(())
}
