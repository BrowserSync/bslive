use crate::inject_opts::{BsLiveStrings, InjectDefinition, InjectOpts, Known};

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
            Known::BsLive(BsLiveStrings::Connector),
            Known::UnknownNamed(String::from("oops")),
            Known::Def(InjectDefinition {
                name: "abc".to_string(),
                before: "</head>".to_string(),
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
        inject: InjectOpts::Items(vec![Known::UnknownNamed(String::from("anything:else"))]),
    };
    let actual: Result<A, _> = serde_yaml::from_str(input);
    assert_eq!(actual.unwrap().inject, expected.inject);
}
