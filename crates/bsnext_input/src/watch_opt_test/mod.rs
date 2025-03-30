use crate::route::{DebounceDuration, FilterKind, Spec, SpecOpts};
use crate::watch_opts::WatchOpts;

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
            filter: Some(FilterKind::StringDefault("**/*.css".into())),
            ignore: None,
            run: None,
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
            filter: Some(FilterKind::StringDefault("**/*.css".into())),
            ignore: None,
            run: None,
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
            ignore: None,
            run: None,
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
            ignore: None,
            run: None,
        }),
    });
    let actual: WatchOpts = serde_yaml::from_str(input).unwrap();
    assert_eq!(actual, expected);
}
