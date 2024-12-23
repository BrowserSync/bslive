use crate::pretty::PrettyPrint;
use crate::OutputWriter;
use bsnext_dto::internal::StartupEvent;
use bsnext_dto::{
    ExternalEventsDTO, GetServersMessageResponseDTO, ServerDTO, ServerIdentityDTO,
    ServersChangedDTO,
};
use std::io::BufWriter;

fn iden_1() -> ServerIdentityDTO {
    ServerIdentityDTO::Address {
        bind_address: "0.0.0.0:3000".to_string(),
    }
}
fn server_1() -> ServerDTO {
    ServerDTO {
        id: "abc".to_string(),
        identity: iden_1(),
        socket_addr: "0.0.0.0:3000".to_string(),
    }
}
// fn iden_2() -> IdentityDTO {
//     IdentityDTO::Address {
//         bind_address: "0.0.0.0:4000".to_string(),
//     }
// }
fn server_2() -> ServerDTO {
    ServerDTO {
        id: "abcdef".to_string(),
        identity: iden_1(),
        socket_addr: "0.0.0.0:4000".to_string(),
    }
}

fn exec(evt: &ExternalEventsDTO) -> anyhow::Result<String> {
    let mut writer = BufWriter::new(Vec::new());
    PrettyPrint.handle_external_event(&mut writer, &evt)?;
    Ok(String::from_utf8(writer.into_inner()?).unwrap())
}

fn exec_startup(evt: &StartupEvent) -> anyhow::Result<(String, String)> {
    let mut writer = BufWriter::new(Vec::new());
    PrettyPrint.handle_startup_event(&mut writer, &evt)?;
    let raw = String::from_utf8(writer.into_inner()?).unwrap();
    let stripped = strip_ansi_escapes::strip(&raw);
    let stripped_str = std::str::from_utf8(&stripped).unwrap().to_string();
    Ok((raw, stripped_str))
}

#[test]
fn test_servers_started() -> anyhow::Result<()> {
    let evt = ExternalEventsDTO::ServersChanged(ServersChangedDTO {
        servers_resp: GetServersMessageResponseDTO {
            servers: vec![server_1(), server_2()],
        },
    });
    let actual = exec(&evt).unwrap();
    insta::assert_snapshot!(actual);
    Ok(())
}

#[test]
fn test_startup_success() -> anyhow::Result<()> {
    let evt = StartupEvent::Started;
    let (actual, stripped) = exec_startup(&evt).unwrap();
    insta::assert_snapshot!(&actual);
    insta::assert_snapshot!(stripped);
    Ok(())
}
