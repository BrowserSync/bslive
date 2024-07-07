use crate::builtin_strings::BuiltinStrings;
use crate::inject_opts::{InjectOpts, Injection};
use crate::inject_replacement::{InjectReplacement, Pos};

#[test]
fn test_inject_opts_bool() {
    #[derive(serde::Deserialize)]
    struct A {
        inject: InjectOpts,
    }
    let input = r#"
    inject: false
    "#;
    let expected = A {
        inject: InjectOpts::Bool(false),
    };
    let actual: A = serde_yaml::from_str(input).unwrap();
    assert_eq!(actual.inject, expected.inject);
}

#[test]
fn test_inject_opts_list() {
    #[derive(Debug, serde::Deserialize)]
    struct A {
        inject: InjectOpts,
    }
    let input = r#"
    inject:
    - bslive:connector
    - oops
    - name: "abc"
      before: </head>
      content: <!-- lol -->
    "#;
    let expected = A {
        inject: InjectOpts::Items(vec![
            Injection::BsLive(BuiltinStrings::Connector),
            Injection::UnknownNamed(String::from("oops")),
            Injection::Replacement(InjectReplacement {
                pos: Pos::Before("</head>".to_string()),
                content: "<!-- lol -->".to_string(),
            }),
        ]),
    };
    let actual: Result<A, _> = serde_yaml::from_str(input);
    assert_eq!(actual.unwrap().inject, expected.inject);
}
#[test]
fn test_inject_custom_list() {
    #[derive(Debug, serde::Deserialize)]
    struct A {
        inject: InjectOpts,
    }
    let input = r#"
    inject:
      - anything:else
    "#;
    let expected = A {
        inject: InjectOpts::Items(vec![Injection::UnknownNamed(String::from("anything:else"))]),
    };
    let actual: Result<A, _> = serde_yaml::from_str(input);
    assert_eq!(actual.unwrap().inject, expected.inject);
}
