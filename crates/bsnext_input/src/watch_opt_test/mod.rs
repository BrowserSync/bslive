use crate::route::{
    DebounceDuration, PathPattern, RunAll, RunAllOpts, RunOptItem, RunSeq, SeqOpts, ShRunOptItem,
    WatchSpec,
};
use crate::watch_opts::WatchOpts;

#[test]
fn test_watch_opts_debounce() {
    let input = r#"
    debounce:
      ms: 200
    only: "**/*.css"
    "#;
    let expected = WatchOpts::Spec(WatchSpec {
        debounce: Some(DebounceDuration::Ms(200)),
        only: Some(PathPattern::StringDefault("**/*.css".into())),
        ignore: None,
        run: None,
        before: None,
    });
    let actual: WatchOpts = serde_yaml::from_str(input).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn test_watch_opts_inline_filter() {
    let input = r#"
    only: "**/*.css"
    "#;
    let expected = WatchOpts::Spec(WatchSpec {
        debounce: None,
        only: Some(PathPattern::StringDefault("**/*.css".into())),
        ignore: None,
        run: None,
        before: None,
    });
    let actual: WatchOpts = serde_yaml::from_str(input).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn test_watch_opts_explicit_filter_ext() {
    let input = r#"
    only:
      ext: "css"
    "#;
    let expected = WatchOpts::Spec(WatchSpec {
        debounce: None,
        only: Some(PathPattern::Extension {
            ext: "css".to_string(),
        }),
        ignore: None,
        run: None,
        before: None,
    });
    let actual: WatchOpts = serde_yaml::from_str(input).unwrap();
    assert_eq!(actual, expected);
}
#[test]
fn test_watch_opts_explicit_filter_glob() {
    let input = r#"
    only:
      glob: "**/*.css"
    "#;
    let expected = WatchOpts::Spec(WatchSpec {
        debounce: None,
        only: Some(PathPattern::Glob {
            glob: "**/*.css".into(),
        }),
        ignore: None,
        run: None,
        before: None,
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
    let expected = WatchSpec {
        run: Some(vec![
            RunOptItem::Sh(ShRunOptItem::new("echo 1")),
            RunOptItem::Sh(ShRunOptItem::new("echo 2")),
            RunOptItem::Sh(ShRunOptItem::new("echo 3")),
        ]),
        ..Default::default()
    };
    let actual: WatchSpec = serde_yaml::from_str(input).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_watch_opts_run_all() {
    let input = r#"
    run:
      - all:
         - sh: echo 1
         - sh: echo 2
         - sh: echo 3
    "#;
    let expected = WatchSpec {
        run: Some(vec![RunOptItem::All(RunAll::new(vec![
            RunOptItem::Sh(ShRunOptItem::new("echo 1")),
            RunOptItem::Sh(ShRunOptItem::new("echo 2")),
            RunOptItem::Sh(ShRunOptItem::new("echo 3")),
        ]))]),
        ..Default::default()
    };
    let actual: WatchSpec = serde_yaml::from_str(input).unwrap();
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
    let expected = WatchSpec {
        run: Some(vec![
            RunOptItem::Sh(ShRunOptItem::new("echo 1")),
            RunOptItem::All(RunAll::new(vec![
                RunOptItem::Sh(ShRunOptItem::new("echo 2")),
                RunOptItem::Sh(ShRunOptItem::new("echo 3")),
                RunOptItem::Sh(ShRunOptItem::new("echo 4")),
            ])),
        ]),
        ..Default::default()
    };
    let actual: WatchSpec = serde_yaml::from_str(input).unwrap();
    assert_eq!(expected, actual);
}

#[test]
fn test_watch_opts_run_all_max_concurrency() {
    let input = r#"
    run:
      - sh: echo 1
      - all:
         - sh: echo 2
         - sh: echo 3
         - sh: echo 4
        opts:
          max: 10
    "#;
    let expected = WatchSpec {
        run: Some(vec![
            RunOptItem::Sh(ShRunOptItem::new("echo 1")),
            RunOptItem::All(RunAll::with_opts(
                vec![
                    RunOptItem::Sh(ShRunOptItem::new("echo 2")),
                    RunOptItem::Sh(ShRunOptItem::new("echo 3")),
                    RunOptItem::Sh(ShRunOptItem::new("echo 4")),
                ],
                RunAllOpts::max(10),
            )),
        ]),
        ..Default::default()
    };
    let actual: WatchSpec = serde_yaml::from_str(input).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn test_watch_opts_run_seq_exit() {
    let input = r#"
    run:
      - seq:
         - sh: echo 2
         - sh: echo 3
         - sh: echo 4
        opts:
          exit_on_fail: false
    "#;
    let expected = WatchSpec {
        run: Some(vec![RunOptItem::Seq(RunSeq::with_opts(
            vec![
                RunOptItem::Sh(ShRunOptItem::new("echo 2")),
                RunOptItem::Sh(ShRunOptItem::new("echo 3")),
                RunOptItem::Sh(ShRunOptItem::new("echo 4")),
            ],
            SeqOpts {
                exit_on_fail: false,
            },
        ))]),
        ..Default::default()
    };
    let actual: WatchSpec = serde_yaml::from_str(input).unwrap();
    assert_eq!(actual, expected);
}
