use crate::route::{DebounceDuration, FilterKind, RunOpt, RunOptItem, ShRunOptItem, Spec};
use crate::watch_opts::WatchOpts;

#[test]
fn test_watch_opts_debounce() {
    let input = r#"
    debounce:
      ms: 200
    filter: "**/*.css"
    "#;
    let expected = WatchOpts::Spec(Spec {
        debounce: Some(DebounceDuration::Ms(200)),
        filter: Some(FilterKind::StringDefault("**/*.css".into())),
        ignore: None,
        run: None,
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
        debounce: None,
        filter: Some(FilterKind::StringDefault("**/*.css".into())),
        ignore: None,
        run: None,
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
        debounce: None,
        filter: Some(FilterKind::Extension {
            ext: "css".to_string(),
        }),
        ignore: None,
        run: None,
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
        debounce: None,
        filter: Some(FilterKind::Glob {
            glob: "**/*.css".into(),
        }),
        ignore: None,
        run: None,
    });
    let actual: WatchOpts = serde_yaml::from_str(input).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_watch_opts_run_seq() {
    let input = r#"
    run:
      - sh: echo 1
      - sh: echo 2
      - sh: echo 3
    "#;
    let expected = Spec {
        run: Some(RunOpt::Seq(vec![
            RunOptItem::Sh(ShRunOptItem::new("echo 1")),
            RunOptItem::Sh(ShRunOptItem::new("echo 2")),
            RunOptItem::Sh(ShRunOptItem::new("echo 3")),
        ])),
        ..Default::default()
    };
    let actual: Spec = serde_yaml::from_str(input).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_watch_opts_run_all() {
    let input = r#"
    run:
      all:
        - sh: echo 1
        - sh: echo 2
        - sh: echo 3
    "#;
    let expected = Spec {
        run: Some(RunOpt::All {
            all: vec![
                RunOptItem::Sh(ShRunOptItem::new("echo 1")),
                RunOptItem::Sh(ShRunOptItem::new("echo 2")),
                RunOptItem::Sh(ShRunOptItem::new("echo 3")),
            ],
        }),
        ..Default::default()
    };
    let actual: Spec = serde_yaml::from_str(input).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_watch_opts_run_all_nested() {
    let input = r#"
    run:
      - sh: echo 1
      - all:
        - sh: echo 2
        - sh: echo 3
        - sh: echo 4
    "#;
    let expected = Spec {
        run: Some(RunOpt::Seq(vec![
            RunOptItem::Sh(ShRunOptItem::new("echo 1")),
            RunOptItem::All {
                all: vec![
                    RunOptItem::Sh(ShRunOptItem::new("echo 2")),
                    RunOptItem::Sh(ShRunOptItem::new("echo 3")),
                    RunOptItem::Sh(ShRunOptItem::new("echo 4")),
                ],
            },
        ])),
        ..Default::default()
    };
    let actual: Spec = serde_yaml::from_str(input).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn error_handled_with_route() {
    let input = r#"
      run:
        - sh: 'echo 1'
    "#;
    let actual: Spec = serde_yaml::from_str(input).unwrap();
    // assert_eq!(expected, actual);
    dbg!(&actual);
}
