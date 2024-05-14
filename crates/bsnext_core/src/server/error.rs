use std::net::SocketAddr;
use thiserror::Error;

// We derive `thiserror::Error`
#[derive(serde::Serialize, serde::Deserialize, Debug, Error)]
pub enum ServerError {
    // The `#[from]` attribute generates `From<JsonRejection> for ApiError`
    // implementation. See `thiserror` docs for more information
    #[error("address in use {socket_addr}")]
    AddrInUse { socket_addr: SocketAddr },
    #[error("invalid bind address: {addr_parse_error}")]
    InvalidAddress { addr_parse_error: String },
    #[error("could not determine the reason")]
    Unknown,
    #[error("server was closed")]
    Closed,
}

#[test]
fn test_api_error() {
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    struct Output {
        error: ServerError,
    }

    let e = ServerError::AddrInUse {
        socket_addr: "127.0.0.1:3000".parse().unwrap(),
    };

    let output = Output { error: e };
    let _as_yaml = serde_yaml::to_string(&output).unwrap();
    let _as_json = serde_json::to_string_pretty(&output).unwrap();
    // println!("{}", as_yaml);
    // println!("{}", as_json);

    let input = r#"
error: !AddrInUse
  socket_addr: 127.0.0.1:3000
    "#;
    let _input: Output = serde_yaml::from_str(&input).unwrap();
}
