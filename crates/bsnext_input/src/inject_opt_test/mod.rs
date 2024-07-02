use crate::inject_opts::InjectOpts;

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
    #[derive(serde::Deserialize)]
    struct A {
        inject: InjectOpts,
    }
    let input = r#"
    inject:
    - abc
    - bslive:connector
    "#;
    let expected = A {
        inject: InjectOpts::Items(vec!["abc".to_string(), "bslive:connector".to_string()]),
    };
    let actual: A = serde_yaml::from_str(input).unwrap();
    assert_eq!(actual.inject, expected.inject);
}
