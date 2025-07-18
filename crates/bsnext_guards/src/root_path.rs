use serde::{de, Deserialize, Deserializer, Serializer};
use std::path::PathBuf;
use std::str::FromStr;

#[derive(Debug, PartialEq, Hash, Clone)]
pub struct RootPath {
    inner: String,
}

#[derive(Debug, PartialEq, Hash, Clone, thiserror::Error)]
pub enum RootPathError {
    #[error("must start with forward slash")]
    MissingSlash,
}

impl FromStr for RootPath {
    type Err = RootPathError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::try_new(s)
    }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<RootPath, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    FromStr::from_str(&s).map_err(de::Error::custom)
}

pub fn serialize<S>(x: &RootPath, s: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    s.serialize_str(x.inner.as_str())
}

impl RootPath {
    pub fn as_pb(&self) -> PathBuf {
        PathBuf::from(&self.inner)
    }
    pub fn as_str(&self) -> &str {
        self.inner.as_str()
    }
    pub fn try_new<A: AsRef<str>>(input: A) -> Result<Self, RootPathError> {
        let str = input.as_ref();
        let is_path = str.starts_with("/");
        if !is_path {
            return Err(RootPathError::MissingSlash);
        }
        Ok(Self {
            inner: String::from(str),
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_verify() -> anyhow::Result<()> {
        let input = "abc/one";
        let actual = RootPath::try_new(input).unwrap_err();
        assert_eq!(actual, RootPathError::MissingSlash);
        let input = "/abc/one";
        let _actual = RootPath::try_new(input).unwrap();
        Ok(())
    }
}
