use crate::route::{DebounceDuration, FilterKind, Spec, SpecOpts, WatchOpts};

#[test]
fn test_watch_opts_debounce() {
    let input = r#"
    debounce:
      ms: 200
    filter: "**/*.css"
    "#;
    let expected = WatchOpts::Spec(Spec {
        opts: Some(SpecOpts {
            debounce: Some(DebounceDuration::Ms(200)),
            filter: Some(FilterKind::StringGlob("**/*.css".into())),
        }),
    });
    let actual: WatchOpts = serde_yaml::from_str(input).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn test_watch_opts_inline_filter() {
    let input = r#"
    filter: "**/*.css"
    "#;
    let expected = WatchOpts::Spec(Spec {
        opts: Some(SpecOpts {
            debounce: None,
            filter: Some(FilterKind::StringGlob("**/*.css".into())),
        }),
    });
    let actual: WatchOpts = serde_yaml::from_str(input).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn test_watch_opts_explicit_filter_ext() {
    let input = r#"
    filter:
      ext: "css"
    "#;
    let expected = WatchOpts::Spec(Spec {
        opts: Some(SpecOpts {
            debounce: None,
            filter: Some(FilterKind::Extension {
                ext: "css".to_string(),
            }),
        }),
    });
    let actual: WatchOpts = serde_yaml::from_str(input).unwrap();
    assert_eq!(actual, expected);
}
#[test]
fn test_watch_opts_explicit_filter_glob() {
    let input = r#"
    filter:
      glob: "**/*.css"
    "#;
    let expected = WatchOpts::Spec(Spec {
        opts: Some(SpecOpts {
            debounce: None,
            filter: Some(FilterKind::Glob {
                glob: "**/*.css".into(),
            }),
        }),
    });
    let actual: WatchOpts = serde_yaml::from_str(input).unwrap();
    assert_eq!(actual, expected);
}
