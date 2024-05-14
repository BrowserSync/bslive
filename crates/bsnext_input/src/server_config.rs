use crate::route::{Route, Watcher};
use crate::{rand_word, PortError};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct ServerConfig {
    #[serde(flatten)]
    pub identity: Identity,
    #[serde(default)]
    pub routes: Vec<Route>,
    #[serde(default)]
    pub watchers: Vec<Watcher>,
}

#[derive(
    Debug, Ord, PartialOrd, PartialEq, Hash, Eq, Clone, serde::Deserialize, serde::Serialize,
)]
#[serde(untagged)]
pub enum Identity {
    Both { name: String, bind_address: String },
    Address { bind_address: String },
    Named { name: String },
}

impl Default for Identity {
    fn default() -> Self {
        Self::named()
    }
}

impl Identity {
    pub fn named() -> Self {
        Self::Named { name: rand_word() }
    }
    pub fn address<A: AsRef<str>>(a: A) -> Self {
        Self::Address {
            bind_address: a.as_ref().to_string(),
        }
    }
    /// if a port was provided, try to use it by validating it first
    /// otherwise default to a named identity
    pub fn from_port_or_named(port: Option<u16>) -> Result<Self, PortError> {
        let result = port.map(|port| {
            SocketAddr::from_str(&format!("0.0.0.0:{port}"))
                .map_err(|err| PortError::InvalidPort { port, err })
        });
        match result {
            None => Ok(Identity::named()),
            Some(Ok(addr)) => Ok(Identity::address(addr.to_string())),
            Some(Err(err)) => Err(err),
        }
    }

    pub fn as_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }
}

#[test]
fn server_config_as_enum() {
    #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
    struct C {
        servers: Vec<ServerConfig>,
    }
    let input = r#"
servers:
    - bind_address: 127.0.0.1:3000
    - name: server_1
    - name: server_2
      bind_address: 127.0.0.1:3001
      routes:
        - path: /
          dir: .
    "#;
    let c: C = serde_yaml::from_str(input).unwrap();

    // let baseline = ServerConfigInput::Named { name: "server_1".into() };
    let baseline = Identity::Address {
        bind_address: "127.0.0.1x:3000".into(),
    };
    for x in &c.servers {
        assert_eq!(x.identity == baseline, false);
    }
}
