use crate::client_config::ClientConfig;
use crate::playground::Playground;
use crate::route::{Route, Watcher};
use crate::{rand_word, PortError};
use serde::{de, Deserializer};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::net::SocketAddr;
use std::str::FromStr;

#[derive(Debug, Default, Clone, serde::Deserialize, serde::Serialize)]
pub struct ServerConfig {
    #[serde(flatten)]
    pub identity: ServerIdentity,
    #[serde(default)]
    pub routes: Vec<Route>,
    #[serde(default)]
    pub watchers: Vec<Watcher>,
    #[serde(default)]
    pub playground: Option<Playground>,
    #[serde(default)]
    pub clients: ClientConfig,
}

impl ServerConfig {
    ///
    /// All regular routes, plus dynamically added ones (for example, through a playground)
    ///
    pub fn combined_routes(&self) -> Vec<Route> {
        let routes = self.routes.clone();
        match &self.playground {
            None => self.routes.clone(),
            Some(playground) => {
                let mut pg_routes = playground.as_routes();
                pg_routes.extend(routes);
                pg_routes
            }
        }
    }
    pub fn raw_routes(&self) -> &[Route] {
        &self.routes
    }

    pub fn from_route(r: Route) -> Self {
        Self {
            routes: vec![r],
            ..std::default::Default::default()
        }
    }
}

#[derive(
    Debug, Ord, PartialOrd, PartialEq, Hash, Eq, Clone, serde::Deserialize, serde::Serialize,
)]
#[serde(untagged)]
pub enum ServerIdentity {
    PortNamed {
        #[serde(deserialize_with = "deserialize_port")]
        port: u16,
        name: String,
    },
    Both {
        name: String,
        bind_address: String,
    },
    Address {
        bind_address: String,
    },
    Named {
        name: String,
    },
    Port {
        #[serde(deserialize_with = "deserialize_port")]
        port: u16,
    },
}

fn deserialize_port<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: Deserializer<'de>,
{
    struct PortVisitor;

    impl<'de> de::Visitor<'de> for PortVisitor {
        type Value = u16;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a string or integer representing a valid port number")
        }

        fn visit_u64<E>(self, value: u64) -> Result<u16, E>
        where
            E: de::Error,
        {
            if value <= u16::MAX as u64 {
                Ok(value as u16)
            } else {
                Err(E::custom(format!("port number out of range: {}", value)))
            }
        }

        fn visit_str<E>(self, value: &str) -> Result<u16, E>
        where
            E: de::Error,
        {
            value
                .parse::<u16>()
                .map_err(|_| E::custom(format!("invalid port number: {}", value)))
        }
    }

    deserializer.deserialize_any(PortVisitor)
}

impl Default for ServerIdentity {
    fn default() -> Self {
        Self::named()
    }
}

impl ServerIdentity {
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
            None => Ok(ServerIdentity::named()),
            Some(Ok(addr)) => Ok(ServerIdentity::address(addr.to_string())),
            Some(Err(err)) => Err(err),
        }
    }

    pub fn as_id(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub fn is_named(&self, pred_name: &str) -> bool {
        match self {
            ServerIdentity::Both { name, .. } => name == pred_name,
            ServerIdentity::Address { .. } => false,
            ServerIdentity::Named { name } => name == pred_name,
            ServerIdentity::Port { .. } => false,
            ServerIdentity::PortNamed { name, .. } => name == pred_name,
        }
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
    let baseline = ServerIdentity::Address {
        bind_address: "127.0.0.1x:3000".into(),
    };
    for x in &c.servers {
        assert_eq!(x.identity == baseline, false);
    }
}

#[test]
fn with_port() {
    #[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
    struct C {
        servers: Vec<ServerConfig>,
    }
    let input = r#"
servers:
    - port: 3000
    - port: "3001"
    "#;
    let c: C = serde_yaml::from_str(input).unwrap();
    let expected = ServerIdentity::Port { port: 3000 };
    let first = c.servers.first().unwrap();
    assert_eq!(first.identity, expected);

    let second = c.servers.get(1).unwrap();
    let second_expected = ServerIdentity::Port { port: 3001 };
    assert_eq!(second.identity, second_expected);
}
