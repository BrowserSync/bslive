use bsnext_dto::internal::ServerError;

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
