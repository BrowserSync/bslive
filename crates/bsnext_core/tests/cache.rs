use bsnext_core::server::router::common::{from_yaml_blocking, header_pairs};

#[test]
fn test_cache_query_param() -> Result<(), anyhow::Error> {
    let input = r#"
servers:
  - name: cache_defaults
    routes:
      - path: /
        raw: hello world!
    "#;

    let (parts1, _, _) = from_yaml_blocking(input, "/")?;
    let pairs = header_pairs(&parts1);

    let control = vec![
        ("content-type", "text/plain"),
        ("content-length", "12"),
        ("cache-control", "no-store, no-cache, must-revalidate"),
        ("pragma", "no-cache"),
        ("expires", "0"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect::<Vec<(String, String)>>();

    assert_eq!(pairs, control);

    let (parts1, _, _) = from_yaml_blocking(input, "/?bslive.cache=default")?;
    let pairs = header_pairs(&parts1);
    let expected = vec![("content-type", "text/plain"), ("content-length", "12")]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<Vec<(String, String)>>();

    assert_eq!(pairs, expected);

    Ok(())
}
#[test]
fn test_cache_query_param_overrides_main() -> Result<(), anyhow::Error> {
    let input = r#"
servers:
  - name: cache_defaults
    routes:
      - path: /abc
        raw: hello world!
        cache: default
    "#;

    let (parts1, _, _) = from_yaml_blocking(input, "/abc")?;
    let pairs = header_pairs(&parts1);

    let control = vec![("content-type", "text/plain"), ("content-length", "12")]
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect::<Vec<(String, String)>>();

    assert_eq!(pairs, control);

    let (parts1, _, _) = from_yaml_blocking(input, "/abc?bslive.cache=prevent")?;
    let pairs = header_pairs(&parts1);
    let expected = vec![
        ("content-type", "text/plain"),
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
