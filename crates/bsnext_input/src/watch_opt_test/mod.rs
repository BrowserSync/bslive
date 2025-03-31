use crate::route::{DebounceDuration, FilterKind, RunOpt, RunOptItem, Spec, SpecOpts};
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
    dbg!(&actual);
    // assert_eq!(actual, expected);
}

#[test]
fn test_watch_opts_run_seq() {
    let input = r#"
    run:
      - sh: echo 1
      - sh: echo 2
      - sh: echo 3
    "#;
    let expected = SpecOpts {
        run: Some(RunOpt::Seq(vec![
            RunOptItem::Sh {
                sh: "echo 1".to_string(),
            },
            RunOptItem::Sh {
                sh: "echo 2".to_string(),
            },
            RunOptItem::Sh {
                sh: "echo 3".to_string(),
            },
        ])),
        ..Default::default()
    };
    let actual: SpecOpts = serde_yaml::from_str(input).unwrap();
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
    let expected = SpecOpts {
        run: Some(RunOpt::All {
            all: vec![
                RunOptItem::Sh {
                    sh: "echo 1".to_string(),
                },
                RunOptItem::Sh {
                    sh: "echo 2".to_string(),
                },
                RunOptItem::Sh {
                    sh: "echo 3".to_string(),
                },
            ],
        }),
        ..Default::default()
    };
    let actual: SpecOpts = serde_yaml::from_str(input).unwrap();
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
    let expected = SpecOpts {
        run: Some(RunOpt::Seq(vec![
            RunOptItem::Sh {
                sh: "echo 1".to_string(),
            },
            RunOptItem::All {
                all: vec![
                    RunOptItem::Sh {
                        sh: "echo 2".to_string(),
                    },
                    RunOptItem::Sh {
                        sh: "echo 3".to_string(),
                    },
                    RunOptItem::Sh {
                        sh: "echo 4".to_string(),
                    },
                ],
            },
        ])),
        ..Default::default()
    };
    let actual: SpecOpts = serde_yaml::from_str(input).unwrap();
    assert_eq!(expected, actual);
}
