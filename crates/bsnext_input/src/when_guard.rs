#[derive(Debug, Default, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum WhenGuard {
    #[default]
    Always,
    Never,
    Query {
        query: Vec<HasGuard>,
    },
}

#[test]
fn test_when_guard() {
    use insta::assert_debug_snapshot;
    #[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
    struct A {
        when: WhenGuard,
    }
    let input = r#"
    when:
      query:
        - not.has: "here"
    "#;
    let when: A = serde_yaml::from_str(input).expect("test");
    assert_debug_snapshot!(&when);
}

#[test]
fn test_when_guard_has() {
    use insta::assert_debug_snapshot;
    #[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
    struct A {
        when: WhenGuard,
    }
    let input = r#"
    when:
      query:
        - has: "here"
    "#;
    let when: A = serde_yaml::from_str(input).expect("test");
    assert_debug_snapshot!(&when);
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum HasGuard {
    /// A direct, exact match
    Is { is: String },
    /// Contains the substring
    Has { has: String },
    /// Contains the substring
    NotHas {
        #[serde(rename = "not.has")]
        not_has: String,
    },
}
