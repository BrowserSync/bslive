use crate::builtin_strings::BuiltinStrings::Connector;
use crate::builtin_strings::{BuiltinStringDef, BuiltinStrings};
use crate::inject_addition::{AdditionPosition, InjectAddition};
use crate::inject_opts::{InjectOpts, Injection, InjectionItem, UnknownStringDef};
use crate::inject_replacement::{InjectReplacement, Pos};
use bsnext_guards::path_matcher::PathMatcher;
use bsnext_guards::MatcherList;

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
    - name: bslive:connector
    - name: oops
    - before: </head>
      content: <!-- lol -->
    "#;
    let expected = A {
        inject: InjectOpts::Items(vec![
            InjectionItem {
                inner: Injection::BsLive(BuiltinStringDef {
                    name: BuiltinStrings::Connector,
                }),
                only: None,
            },
            InjectionItem {
                inner: Injection::UnknownNamed(UnknownStringDef {
                    name: "oops".to_string(),
                }),
                only: None,
            },
            InjectionItem {
                inner: Injection::Replacement(InjectReplacement {
                    pos: Pos::Before("</head>".to_string()),
                    content: "<!-- lol -->".to_string(),
                }),
                only: None,
            },
        ]),
    };
    let actual: Result<A, _> = serde_yaml::from_str(input);
    assert_eq!(actual.unwrap().inject, expected.inject);
}
#[test]
fn test_inject_builtin() {
    #[derive(Debug, serde::Deserialize)]
    struct A {
        inject: InjectOpts,
    }
    let input = r#"
    inject:
      - name: bslive:connector
      - append: 'other'
    "#;
    let expected = A {
        inject: InjectOpts::Items(vec![
            InjectionItem {
                inner: Injection::BsLive(BuiltinStringDef { name: Connector }),
                only: None,
            },
            InjectionItem {
                inner: Injection::Addition(InjectAddition {
                    addition_position: AdditionPosition::Append(String::from("other")),
                }),
                only: None,
            },
        ]),
    };
    let actual: Result<A, _> = serde_yaml::from_str(input);
    assert_eq!(actual.unwrap().inject, expected.inject);
}

#[test]
fn test_inject_replace() {
    #[derive(Debug, serde::Deserialize)]
    struct A {
        inject: InjectOpts,
    }
    let input = r#"
    inject:
      - replace: Basic
        content: huh?
      - before: </body>
        content: </BODY>
      - after: <html>
        content: woop
    "#;
    let expected = A {
        inject: InjectOpts::Items(vec![
            InjectionItem {
                inner: Injection::Replacement(InjectReplacement {
                    pos: Pos::Replace("Basic".to_string()),
                    content: "huh?".to_string(),
                }),
                only: None,
            },
            InjectionItem {
                inner: Injection::Replacement(InjectReplacement {
                    pos: Pos::Before("</body>".to_string()),
                    content: "</BODY>".to_string(),
                }),
                only: None,
            },
            InjectionItem {
                inner: Injection::Replacement(InjectReplacement {
                    pos: Pos::After("<html>".to_string()),
                    content: "woop".to_string(),
                }),
                only: None,
            },
        ]),
    };
    let actual: Result<A, _> = serde_yaml::from_str(input);
    assert_eq!(actual.unwrap().inject, expected.inject);
}

#[test]
fn test_inject_replace_single() {
    #[derive(Debug, serde::Deserialize)]
    struct A {
        inject: InjectOpts,
    }
    let input = r#"
    inject:
        replace: Basic
        content: huh?
    "#;
    let expected = A {
        inject: InjectOpts::Item(InjectionItem {
            inner: Injection::Replacement(InjectReplacement {
                pos: Pos::Replace("Basic".to_string()),
                content: "huh?".to_string(),
            }),
            only: None,
        }),
    };
    let actual: Result<A, _> = serde_yaml::from_str(input);
    assert_eq!(actual.unwrap().inject, expected.inject);
}

#[test]
fn test_inject_append_prepend() {
    #[derive(Debug, serde::Deserialize)]
    struct A {
        inject: InjectOpts,
    }
    let input = r#"
    inject:
      - append: lol
      - prepend: lol2
    "#;
    let expected = A {
        inject: InjectOpts::Items(vec![
            InjectionItem {
                inner: Injection::Addition(InjectAddition {
                    addition_position: AdditionPosition::Append("lol".to_string()),
                }),
                only: None,
            },
            InjectionItem {
                inner: Injection::Addition(InjectAddition {
                    addition_position: AdditionPosition::Prepend("lol2".to_string()),
                }),
                only: None,
            },
        ]),
    };
    let actual: Result<A, _> = serde_yaml::from_str(input);
    assert_eq!(actual.unwrap().inject, expected.inject);
}

#[test]
fn test_path_matchers() {
    use std::str::FromStr;
    #[derive(Debug, serde::Deserialize)]
    struct A {
        inject: InjectOpts,
    }
    let input = r#"
inject:
    append: lol
    only:
      - /*.css
      - pathname: /*.css
"#;
    let expected = A {
        inject: InjectOpts::Item(InjectionItem {
            inner: Injection::Addition(InjectAddition {
                addition_position: AdditionPosition::Append("lol".to_string()),
            }),
            only: Some(MatcherList::Items(vec![
                PathMatcher::from_str("/*.css").unwrap(),
                PathMatcher::pathname("/*.css"),
            ])),
        }),
    };
    let actual: Result<A, _> = serde_yaml::from_str(input);
    assert_eq!(actual.unwrap().inject, expected.inject);
}

#[test]
fn test_path_matcher_single() {
    use std::str::FromStr;
    #[derive(Debug, serde::Deserialize)]
    struct A {
        inject: InjectOpts,
    }
    let input = r#"
    inject:
        append: lol
        only: /*.css
    "#;
    let expected = A {
        inject: InjectOpts::Item(InjectionItem {
            inner: Injection::Addition(InjectAddition {
                addition_position: AdditionPosition::Append("lol".to_string()),
            }),
            only: Some(MatcherList::Item(PathMatcher::from_str("/*.css").unwrap())),
        }),
    };
    let actual: Result<A, _> = serde_yaml::from_str(input);
    assert_eq!(actual.unwrap().inject, expected.inject);
}
