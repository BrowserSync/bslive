# [DynamicQueryParams]

Dynamically adjust how requests and responses will be handled on the fly, using
query params.

**Features**

- [delay](#delay-example) - simulate a delay in TTFB.
- [cache](#cache-example) - add or remove the headers that Browsersync to control cache

---

## Delay

You can control a simulated delay by appending the query param as seen below.

- Note: Only milliseconds are supported right now.
- Note: If there's a typo, of if the value cannot be converted into a millisecond representation
  no error will be thrown, it will simply be ignored.

**When is this useful?**

You can use it to optionally cause an asset to be delayed in its response

### Delay example

```rust
# use bsnext_core::server::router::common::from_yaml_blocking;
fn main() -> anyhow::Result<()> {
    let req = "/abc?bslive.delay.ms=200";
    let server_yaml = r#"
        servers:
        - name: test
          routes:
            - path: /abc
              html: hello world!
    "#;

    let (parts, body, duration) = from_yaml_blocking(server_yaml, req)?;
    let duration_millis = duration.as_millis();

    assert_eq!(body, "hello world!");
    assert_eq!(parts.status, 200);
    assert!(duration_millis > 200 && duration_millis < 210);
    Ok(())
}
```

### Delay CLI Example

```bash
bslive examples/basic/public -p 3000

# then, in another terminal
curl localhost:3000?bslive.delay.ms=2000
```

### Cache example

The normal behaviour in Browsersync is to add the following HTTP headers to requests in development.

- `cache-control: no-store, no-cache, must-revalidate`
- `pragma: no-cache`
- `expires: 0`

Those indicate that the browser should re-fetch the assets frequently. If you want to override this behavior, you can
provide the query param seen below:

- `?bslive.cache=default` <- this prevents Browsersync from adding any cache headers (defaulting to whatever the browser
  decides)
- `?bslive.cache=prevent` <- this will cause the headers above to be added.

```rust
# use bsnext_core::server::router::common::{from_yaml_blocking, header_pairs};
fn main() -> anyhow::Result<()> {
    let server_yaml = r#"
        servers:
        - name: test
          routes:
            - path: /abc
              html: hello world!
    "#;

    let (parts1, _, _) = from_yaml_blocking(server_yaml, "/abc?bslive.cache=default")?;
    let pairs = header_pairs(&parts1);

    // Note: now the extra 3 headers are present
    let expected = vec![
        ("content-type", "text/html; charset=utf-8"),
        ("content-length", "12")
    ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<Vec<(String, String)>>();

    assert_eq!(pairs, expected);
    Ok(())
}
```

## Cache example, overriding config

At the route-level, you can remove the headers that Browsersync adds to bust caches, by simply putting
`cache: default` at any route-level config.

Then, on a case-by-case basis you can re-enable it.

```rust
# use bsnext_core::server::router::common::{from_yaml_blocking, header_pairs};
fn main() -> anyhow::Result<()> {
    // Note the `cache: default` here. This stops Browsersync adding any headers for cache-busting 
    let server_yaml = r#"
        servers:
        - name: test
          cache: default
          routes:
            - path: /abc
              html: hello world!
    "#;

    // But, now we can re-enable cache-busting on a single URL
    let (parts1, _, _) = from_yaml_blocking(server_yaml, "/abc?bslive.cache=prevent")?;
    let pairs = header_pairs(&parts1);

    // Note: only the 2 headers are present now, otherwise there would be 5
    let expected = vec![
        ("content-type", "text/html; charset=utf-8"),
        ("content-length", "12"),
        ("cache-control", "no-store, no-cache, must-revalidate"),
        ("pragma", "no-cache"),
        ("expires", "0"),
    ]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<Vec<(String, String)>>();

    assert_eq!(pairs, expected);
    Ok(())
}
```