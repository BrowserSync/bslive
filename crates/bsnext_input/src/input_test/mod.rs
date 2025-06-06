use crate::route::{
    CompressionOpts, CorsOpts, DebounceDuration, DelayKind, DelayOpts, FilterKind, Route, Spec,
    Watcher,
};
use crate::watch_opts::WatchOpts;
use crate::Input;
use insta::assert_debug_snapshot;

#[test]
fn test_deserialize() {
    let input = include_str!("../../../../examples/kitchen-sink/bslive.yml");
    let _: Input = serde_yaml::from_str(input).unwrap();
}

#[test]
fn test_deserialize_2() {
    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    struct Config {
        pub items: Vec<Route>,
    }

    let input = r#"
    items:
      - path: /hello.js
        raw: "hello"
        cors: true
        delay:
            ms: 2000
      - path: /hello.js
        json: ["2", "3"]
      - path: /node_modules
        dir: ./node_modules
      - path: /node_modules
        dir: ./node_modules
      - path: /api
        proxy: example.com

        "#;
    let c: Config = serde_yaml::from_str(input).unwrap();
    let first = c.items.get(0).unwrap().to_owned();
    let opts = first.opts.cors.unwrap();
    assert_eq!(opts, CorsOpts::Cors(true));

    let delay_opts = first.opts.delay.unwrap();
    assert_eq!(delay_opts, DelayOpts::Delay(DelayKind::Ms(2000)));
}

#[test]
fn test_deserialize_cors_false() {
    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    struct Config {
        pub items: Vec<Route>,
    }

    let input = r#"
    items:
      - path: /hello.js
        raw: "hello"
        cors: false
        "#;
    let c: Config = serde_yaml::from_str(input).unwrap();
    let first = c.items.get(0).unwrap().to_owned();
    let opts = first.opts.cors.unwrap();
    assert_eq!(opts, CorsOpts::Cors(false));
}

#[test]
fn test_deserialize_compressions_absent() {
    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    struct Config {
        pub items: Vec<Route>,
    }

    let input = r#"
    items:
      - path: /hello.js
        raw: "hello"
        "#;
    let c: Config = serde_yaml::from_str(input).unwrap();
    let first = c.items.get(0).unwrap();
    assert_eq!(first.opts.compression, None);
}

#[test]
fn test_deserialize_compressions_true() {
    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    struct Config {
        pub items: Vec<Route>,
    }

    let input = r#"
    items:
      - path: /hello.js
        raw: "hello"
        compression: true
        "#;
    let c: Config = serde_yaml::from_str(input).unwrap();
    let first = c.items.get(0).unwrap();
    assert_eq!(first.opts.compression, Some(CompressionOpts::Bool(true)));
}
#[test]
fn test_deserialize_compressions_gzip() {
    let input = r#"
      - path: /hello.js
        raw: "hello"
        compression: gzip
      - path: /hello2.js
        raw: "hello"
        compression: br
        "#;
    let c: Vec<Route> = serde_yaml::from_str(input).unwrap();
    assert_debug_snapshot!(c.get(0).unwrap().opts.compression);
    assert_debug_snapshot!(c.get(1).unwrap().opts.compression);
}

#[test]
fn test_com_yaml() -> anyhow::Result<()> {
    #[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
    struct V {
        compression: CompressionOpts,
    }

    let input = r#"
    - compression: true
    - compression: false
    - compression: br
    - compression: gzip
    - compression: zstd
    - compression: deflate
    "#;
    let v: Vec<V> = serde_yaml::from_str(input)?;
    assert_debug_snapshot!(v);
    Ok(())
}

#[test]
fn test_deserialize_3_headers_control() {
    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    struct Config {
        pub items: Vec<Route>,
    }

    let input = r#"
    items:
      - path: /api
        json: [1,2]
        "#;
    let c: Config = serde_yaml::from_str(input).unwrap();
    insta::assert_debug_snapshot!(c)
}

#[test]
fn test_deserialize_3_headers() {
    #[derive(serde::Deserialize, serde::Serialize, Debug)]
    struct Config {
        pub items: Vec<Route>,
    }

    let input = r#"
    items:
      - path: /api
        json: [1,2]
        headers:
            a: b
        "#;
    let c: Config = serde_yaml::from_str(input).unwrap();
    insta::assert_debug_snapshot!(c)
}

#[test]
fn test_deserialize_3() {
    let input = r#"
    path: /hello.js
    dir: "hello"
    cors: true
        "#;
    let _c: Route = serde_yaml::from_str(input).unwrap();
}

#[test]
fn test_deserialize_watch() {
    let input = r#"
    path: /hello.js
    dir: "hello"
    watch: true
        "#;
    let c: Route = serde_yaml::from_str(input).unwrap();
    assert_eq!(c.opts.watch, WatchOpts::Bool(true));
    let input = r#"
    path: /hello.js
    dir: "hello"
    watch: false
        "#;
    let c: Route = serde_yaml::from_str(input).unwrap();
    assert_eq!(c.opts.watch, WatchOpts::Bool(false));
    let input = r#"
    path: /hello.js
    dir: "hello"
    watch: "public/**/*.css"
        "#;
    let c: Route = serde_yaml::from_str(input).unwrap();
    assert_eq!(
        c.opts.watch,
        WatchOpts::InlineGlob("public/**/*.css".into())
    );
    let input = r#"
    path: /hello.js
    dir: "hello"
    watch:
      debounce:
        ms: 2000
        "#;
    let c: Route = serde_yaml::from_str(input).unwrap();
    assert_eq!(
        c.opts.watch,
        WatchOpts::Spec(Spec {
            debounce: Some(DebounceDuration::Ms(2000)),
            filter: None,
            ignore: None,
            run: None,
            before: None,
        })
    );
}

#[test]
fn test_deserialize_server_watch_list() {
    let input = r#"
servers:
- bind_address: 0.0.0.0:4000
  watchers:
    - dir: ./
    - dir: ./other
      debounce:
        ms: 2000
      filter:
        ext: "**/*.css"
"#;
    let c: Input = serde_yaml::from_str(input).unwrap();
    assert_eq!(
        c.servers.get(0).unwrap().watchers,
        vec![
            Watcher {
                dir: Some("./".to_string()),
                dirs: None,
                opts: Some(Spec::default())
            },
            Watcher {
                dir: Some("./other".to_string()),
                dirs: None,
                opts: Some(Spec {
                    debounce: Some(DebounceDuration::Ms(2000)),
                    filter: Some(FilterKind::Extension {
                        ext: "**/*.css".to_string()
                    }),
                    ..Default::default()
                })
            }
        ]
    )
}
#[test]
fn test_deserialize_server_clients_config() {
    let input = r#"
servers:
- bind_address: 0.0.0.0:4000
  clients:
    log: debug
"#;
    let c: Input = serde_yaml::from_str(input).unwrap();
    dbg!(&c);
}
