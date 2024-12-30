use crate::internal::{ChildResult, InternalEvents, InternalEventsDTO};
use crate::{GetActiveServersResponseDTO, ServerIdentityDTO};
use bsnext_input::InputError;
use bsnext_output::OutputWriterTrait;
use std::io::Write;

impl OutputWriterTrait for InternalEvents {
    fn write_json<W: Write>(&self, sink: &mut W) -> anyhow::Result<()> {
        match self {
            InternalEvents::ServersChanged { server_resp, .. } => {
                let as_dto = GetActiveServersResponseDTO::from(server_resp);
                let output = InternalEventsDTO::ServersChanged(as_dto);
                writeln!(sink, "{}", serde_json::to_string(&output)?)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            }
            InternalEvents::InputError(_) => {}
            InternalEvents::StartupError(_) => {}
        }
        Ok(())
    }

    fn write_pretty<W: Write>(&self, sink: &mut W) -> anyhow::Result<()> {
        match self {
            InternalEvents::ServersChanged {
                server_resp: _,
                child_results,
            } => {
                let lines = print_server_updates(child_results);
                for x in lines {
                    if let Err(e) = writeln!(sink, "{x}") {
                        tracing::error!(?e);
                    }
                }
            }
            InternalEvents::InputError(InputError::BsLiveRules(bs_rules)) => {
                let n = miette::GraphicalReportHandler::new();
                let mut inner = String::new();
                n.render_report(&mut inner, bs_rules).expect("write?");
                writeln!(sink, "{}", inner)?;
            }
            InternalEvents::InputError(err) => {
                writeln!(sink, "{}", err)?;
            }
            InternalEvents::StartupError(err) => {
                writeln!(sink, "{}", err)?;
            }
        }
        Ok(())
    }
}

pub fn print_server_updates(evts: &[ChildResult]) -> Vec<String> {
    evts.iter()
        .flat_map(|r| match r {
            ChildResult::Created(created) => {
                vec![format!(
                    "[created] {}",
                    server_display(
                        &ServerIdentityDTO::from(&created.server_handler.identity),
                        &created.server_handler.socket_addr.to_string()
                    ),
                )]
            }
            ChildResult::Stopped(stopped) => {
                vec![format!("[stopped] {}", iden(stopped))]
            }
            ChildResult::CreateErr(errored) => {
                vec![format!(
                    "[server] did not create, reason: {}",
                    errored.server_error
                )]
            }
            ChildResult::Patched(child) => {
                let mut lines = vec![];
                // todo: determine WHICH changes were actually applied (instead of saying everything was patched)
                for x in &child.route_change_set.changed {
                    lines.push(format!(
                        "[patched] {} {:?}",
                        iden(&child.server_handler.identity),
                        x
                    ));
                }
                for x in &child.route_change_set.added {
                    lines.push(format!(
                        "[patched] {} {:?}",
                        iden(&child.server_handler.identity),
                        x
                    ));
                }
                lines
            }
            ChildResult::PatchErr(errored) => {
                vec![format!(
                    "[patch] error {} {} ",
                    iden(&errored.identity),
                    errored.patch_error
                )]
            }
        })
        .collect()
}

pub fn server_display(identity_dto: &ServerIdentityDTO, socket_addr: &str) -> String {
    match &identity_dto {
        ServerIdentityDTO::Both { name, .. } => {
            format!("[server] [{}] http://{}", name, socket_addr)
        }
        ServerIdentityDTO::Address { .. } => {
            format!("[server] http://{}", socket_addr)
        }
        ServerIdentityDTO::Named { name } => {
            format!("[server] [{}] http://{}", name, &socket_addr)
        }
    }
}

pub fn iden(identity_dto: impl Into<ServerIdentityDTO>) -> String {
    match identity_dto.into() {
        ServerIdentityDTO::Both { name, bind_address } => format!("[{name}] {bind_address}"),
        ServerIdentityDTO::Address { bind_address } => bind_address.to_string(),
        ServerIdentityDTO::Named { name } => format!("[{name}]"),
    }
}
