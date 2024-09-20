use crate::route::PathDefError;
use serde::{de, Deserialize, Deserializer, Serializer};
use std::path::{Component, Path, PathBuf};
use std::str::FromStr;

#[derive(Debug, PartialEq, Hash, Clone)]
pub struct PathDef {
    inner: String,
}

impl FromStr for PathDef {
    type Err = PathDefError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<PathDef, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    FromStr::from_str(&s).map_err(de::Error::custom)
}

pub fn serialize<S>(x: &PathDef, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(x.inner.as_str())
}

impl PathDef {
    pub fn as_pb(&self) -> PathBuf {
        PathBuf::from(&self.inner)
    }
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }
    pub fn try_new<A: AsRef<str>>(input: A) -> Result<Self, PathDefError> {
        let str = input.as_ref();
        let is_path = str.starts_with("/");
        let p = Path::new(str);
        let has_star = p.components().any(|c| match c {
            Component::Prefix(_) => false,
            Component::RootDir => false,
            Component::CurDir => false,
            Component::ParentDir => false,
            Component::Normal(n) => n.to_str().is_some_and(|str| str.contains("*")),
        });
        let route = match (is_path, has_star) {
            (true, false) => Ok(Self {
                inner: str.to_string(),
            }),
            (true, true) => Err(PathDefError::ContainsStar),
            (false, _) => Err(PathDefError::StartsWithSlash),
        }?;

        let mut r = matchit::Router::new();
        match r.insert(&route.inner, true) {
            Ok(_) => Ok(route),
            Err(e) => Err(PathDefError::InsertError(e)),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_verify() -> anyhow::Result<()> {
        let input = "/abc/one/*rest";
        let actual = PathDef::try_new(input).unwrap_err();
        assert_eq!(actual, PathDefError::ContainsStar);

        let input = "abc/one";
        let actual = PathDef::try_new(input).unwrap_err();
        assert_eq!(actual, PathDefError::StartsWithSlash);

        let input = "/:";
        let actual = PathDef::try_new(input).unwrap_err();
        assert_eq!(
            actual,
            PathDefError::InsertError(matchit::InsertError::UnnamedParam)
        );

        let input = "/:abc:abc";
        let actual = PathDef::try_new(input).unwrap_err();
        assert_eq!(
            actual,
            PathDefError::InsertError(matchit::InsertError::TooManyParams)
        );

        Ok(())
    }
}
