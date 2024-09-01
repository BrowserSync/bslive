use crate::route::{
    CorsOpts, DebounceDuration, DelayKind, DelayOpts, FilterKind, Route, Spec, SpecOpts, Watcher,
};
use crate::watch_opts::WatchOpts;
use crate::Input;

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
            opts: Some(SpecOpts {
                debounce: Some(DebounceDuration::Ms(2000)),
                filter: None,
            })
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
      debounce_ms: 2000
      filter:
        ext: "**/*.css"
"#;
    let c: Input = serde_yaml::from_str(input).unwrap();
    dbg!(&c);
    assert_eq!(
        c.servers.get(0).unwrap().watchers,
        vec![
            Watcher {
                dir: "./".to_string(),
                debounce_ms: None,
                filter: None,
            },
            Watcher {
                dir: "./other".to_string(),
                debounce_ms: Some(2000),
                filter: Some(FilterKind::Extension {
                    ext: "**/*.css".to_string()
                }),
            }
        ]
    )
}
