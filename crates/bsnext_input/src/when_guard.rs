#[derive(Debug, Default, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum WhenGuard {
    #[default]
    Always,
    Never,
    ExactUri {
        exact_uri: bool,
    },
    Query {
        query: HasGuard,
    },
    Accept {
        accept: HasGuard,
    },
}

#[derive(Debug, Default, PartialEq, Hash, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum WhenBodyGuard {
    #[default]
    Never,
    Json {
        json: JsonGuard,
    },
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum JsonGuard {
    ArrayLast {
        items: String,
        last: Vec<JsonPropGuard>,
    },
    ArrayAny {
        items: String,
        any: Vec<JsonPropGuard>,
    },
    ArrayAll {
        items: String,
        all: Vec<JsonPropGuard>,
    },
    Path(JsonPropGuard),
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum JsonPropGuard {
    PathIs { path: String, is: String },
    PathHas { path: String, has: String },
}

impl JsonPropGuard {
    pub fn path(&self) -> &str {
        match self {
            JsonPropGuard::PathIs { path, .. } => path,
            JsonPropGuard::PathHas { path, .. } => path,
        }
    }
}

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
#[serde(untagged)]
pub enum HasGuard {
    /// A direct, exact match
    Literal(String),
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
        not.has: "here"
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
        has: "here"
    "#;
    let when: A = serde_yaml::from_str(input).expect("test");
    assert_debug_snapshot!(&when);
}
