use crate::route::{RawRoute, Route, RouteKind};
use std::collections::{HashMap, HashSet};
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RoutesManifest {
    inner: HashMap<RouteIdentity, u64>,
}

impl RoutesManifest {
    pub fn changeset_for(&self, other: &Self) -> RouteChangeSet {
        let prev = self.inner.keys().collect::<HashSet<_>>();
        let next = other.inner.keys().collect::<HashSet<_>>();
        let changed_only = prev
            .intersection(&next)
            .filter_map(|id| {
                let old = self.inner.get(*id);
                let new = other.inner.get(*id);
                match (old, new) {
                    (Some(old), Some(new)) if old != new => Some((*id).to_owned()),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        RouteChangeSet {
            added: next.difference(&prev).map(|x| (*x).to_owned()).collect(),
            removed: prev.difference(&next).map(|x| (*x).to_owned()).collect(),
            changed: changed_only,
        }
    }
    pub fn new<A: AsRef<Route>>(routes: &[A]) -> Self {
        Self {
            inner: routes
                .iter()
                .map(|a| {
                    let route = a.as_ref();
                    let mut hasher = DefaultHasher::new();
                    route.hash(&mut hasher);
                    let r2_hash = hasher.finish();
                    let id: RouteIdentity = a.as_ref().into();
                    (id, r2_hash)
                })
                .collect::<HashMap<RouteIdentity, u64>>(),
        }
    }
}

#[derive(Debug, PartialEq, Hash, Eq, Clone)]
pub struct RouteIdentity {
    pub path: String,
    pub kind_str: String,
}

impl From<&Route> for RouteIdentity {
    fn from(value: &Route) -> Self {
        Self {
            path: value.path.as_str().to_string(),
            kind_str: match &value.kind {
                RouteKind::Raw(raw) => match raw {
                    RawRoute::Html { .. } => "RouteKind::Raw::Html",
                    RawRoute::Json { .. } => "RouteKind::Raw::Json",
                    RawRoute::Raw { .. } => "RouteKind::Raw::Raw",
                    RawRoute::Sse { .. } => "RouteKind::Raw::Sse",
                },
                RouteKind::Proxy(_) => "RouteKind::Proxy",
                RouteKind::Dir(_) => "RouteKind::Dir",
            }
            .to_string(),
        }
    }
}
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct RouteChangeSet {
    pub added: Vec<RouteIdentity>,
    pub removed: Vec<RouteIdentity>,
    pub changed: Vec<RouteIdentity>,
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::path_def::PathDef;
    use std::hash::DefaultHasher;
    use std::str::FromStr;

    #[test]
    fn test_route_hash() -> anyhow::Result<()> {
        let r1 = Route {
            path: PathDef::from_str("/")?,
            kind: RouteKind::new_html("hello world!"),
            ..Default::default()
        };
        let r2 = r#"
path: /
html: hello world!
        "#;
        let r2: Route = serde_yaml::from_str(&r2).expect("test");

        let mut hasher = DefaultHasher::new();
        r1.hash(&mut hasher);
        let r1_hash = hasher.finish();

        let mut hasher = DefaultHasher::new();
        r2.hash(&mut hasher);
        let r2_hash = hasher.finish();

        assert_eq!(r1_hash, r2_hash);

        Ok(())
    }

    #[test]
    fn test_route_hash_2() -> anyhow::Result<()> {
        let r1 = r#"
path: /
html: hello world!
        "#;
        let r2 = r#"
path: /api
html: hello world!
        "#;
        let r2_edited = r#"
path: /api
html: hello world
        "#;

        let r1: Route = serde_yaml::from_str(&r1).expect("test");
        let r2: Route = serde_yaml::from_str(&r2).expect("test");
        let r2_edited: Route = serde_yaml::from_str(&r2_edited).expect("test");

        let routes_orig = RoutesManifest::new(&[&r1, &r2]);
        let routes_next = RoutesManifest::new(&[&r1]);
        let routes_next_dup = RoutesManifest::new(&[&r1, &r2]);
        let routes_next_3 = RoutesManifest::new(&[&r1, &r2_edited]);

        let changeset_1 = routes_orig.changeset_for(&routes_next);
        let changeset_2 = routes_orig.changeset_for(&routes_next_dup);
        let changeset_3 = routes_orig.changeset_for(&routes_next_3);

        assert_eq!(
            changeset_1,
            RouteChangeSet {
                added: vec![],
                removed: vec![RouteIdentity {
                    path: "/api".to_string(),
                    kind_str: "RouteKind::Raw::Html".to_string()
                }],
                changed: vec![],
            }
        );

        assert_eq!(
            changeset_1,
            RouteChangeSet {
                added: vec![],
                removed: vec![RouteIdentity {
                    path: "/api".to_string(),
                    kind_str: "RouteKind::Raw::Html".to_string()
                }],
                changed: vec![],
            }
        );

        // dbg!(changeset_1);
        assert_eq!(
            changeset_2,
            RouteChangeSet {
                added: vec![],
                removed: vec![],
                changed: vec![],
            }
        );

        assert_eq!(
            changeset_3,
            RouteChangeSet {
                added: vec![],
                removed: vec![],
                changed: vec![RouteIdentity {
                    path: "/api".to_string(),
                    kind_str: "RouteKind::Raw::Html".to_string()
                }],
            }
        );

        Ok(())
    }
}
