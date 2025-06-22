use crate::archy::archy;
use crate::internal::{ChildResult, InternalEvents, InternalEventsDTO, TaskReportAndTree};
use crate::{GetActiveServersResponseDTO, InputErrorDTO, ServerIdentityDTO};
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
            InternalEvents::InputError(err) => {
                let e = InputErrorDTO::from(err);
                writeln!(sink, "{}", serde_json::to_string(&e)?)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            }
            InternalEvents::StartupError(startup) => {
                let str = startup.to_string();
                let json = serde_json::json!({
                    "_unstable_InternalEvents::StartupError": str
                });
                writeln!(sink, "{}", serde_json::to_string(&json)?)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
            }
            InternalEvents::TaskReport(TaskReportAndTree { report, .. }) => {
                // let tree_str = archy(tree, None);
                let as_json = InternalEventsDTO::TaskReport {
                    id: report.id().to_string(),
                };
                writeln!(sink, "{}", serde_json::to_string(&as_json)?)
                    .map_err(|e| anyhow::anyhow!(e.to_string()))?;
                // let mut e = HashMap::new();
                // every_report(&mut e, &report);

                // let out = runner.as_tree_with_results(&e);
                // let s = archy(&out, None);

                // tracing::trace!("will handle !result.is_ok()");
            }
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
            InternalEvents::TaskReport(TaskReportAndTree { report, tree }) => {
                if report.has_errors() {
                    let s = archy(tree, None);
                    write!(sink, "{}", s)?;
                }
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
                        "[added] {} {:?}",
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
        ServerIdentityDTO::Port { port } => {
            format!("[server] [{}] http://{}", port, &socket_addr)
        }
        ServerIdentityDTO::PortNamed { name, .. } => {
            format!("[server] [{}] http://{}", name, &socket_addr)
        }
    }
}

pub fn iden(identity_dto: impl Into<ServerIdentityDTO>) -> String {
    match identity_dto.into() {
        ServerIdentityDTO::Both { name, bind_address } => format!("[{name}] {bind_address}"),
        ServerIdentityDTO::Address { bind_address } => bind_address.to_string(),
        ServerIdentityDTO::Named { name } => format!("[{name}]"),
        ServerIdentityDTO::Port { port } => format!("[{port}]"),
        ServerIdentityDTO::PortNamed { port, name } => format!("[{name}] {port}"),
    }
}
