use crate::pretty::PrettyPrint;
use crate::OutputWriter;
use bsnext_dto::{
    ExternalEvents, GetServersMessageResponse, IdentityDTO, ServerChange,
    ServerChangeSet, ServerChangeSetItem, ServerDTO, ServersStarted, StartupEvent,
};
use std::io::{BufWriter};

fn iden_1() -> IdentityDTO {
    IdentityDTO::Address {
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
fn iden_2() -> IdentityDTO {
    IdentityDTO::Address {
        bind_address: "0.0.0.0:4000".to_string(),
    }
}
fn server_2() -> ServerDTO {
    ServerDTO {
        id: "abcdef".to_string(),
        identity: iden_1(),
        socket_addr: "0.0.0.0:4000".to_string(),
    }
}

fn exec(evt: &ExternalEvents) -> anyhow::Result<String> {
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
    let evt = ExternalEvents::ServersStarted(ServersStarted {
        servers_resp: GetServersMessageResponse {
            servers: vec![server_1(), server_2()],
        },
        changeset: ServerChangeSet {
            items: vec![
                ServerChangeSetItem {
                    identity: iden_1(),
                    change: ServerChange::Started,
                },
                ServerChangeSetItem {
                    identity: iden_2(),
                    change: ServerChange::Stopped {
                        bind_address: server_2().socket_addr,
                    },
                },
            ],
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
