use crate::start_kind::start_fs;
use crate::start_kind::start_fs::WriteMode;
use bsnext_input::paths::from_paths;
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
use bsnext_input::target::TargetKind;
use bsnext_input::InputError;

#[derive(Debug)]
pub struct StartFromPaths {
    pub paths: Vec<String>,
    pub write_input: bool,
    pub port: Option<u16>,
    pub force: bool,
}

impl SystemStart for StartFromPaths {
    fn input(&self, ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        let identity =
            ServerIdentity::from_port_or_named(self.port).map_err(|e| Box::new(e.into()))?;
        let input = from_paths(&ctx.cwd, &self.paths, identity).map_err(|e| Box::new(e.into()))?;
        let write_mode = if self.force {
            WriteMode::Override
        } else {
            WriteMode::Safe
        };
        if self.write_input {
            let path = start_fs::fs_write_input(&ctx.cwd, &input, TargetKind::Yaml, &write_mode)
                .map_err(|e| Box::new(e.into()))?;
            Ok(SystemStartArgs::PathWithInput { input, path })
        } else {
            Ok(SystemStartArgs::InputOnly { input })
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test() -> anyhow::Result<()> {
        use tempfile::tempdir;
        let tmp_dir = tempdir()?;
        let v = StartFromPaths {
            paths: vec![".".into()],
            write_input: false,
            port: Some(3000),
            force: false,
        };
        let ctx = StartupContext {
            cwd: tmp_dir.path().to_path_buf(),
        };
        let i = v.input(&ctx);
        tmp_dir.close()?;
        let start_args = i.unwrap();
        if let SystemStartArgs::InputOnly { input } = start_args {
            insta::assert_debug_snapshot!(input);
            insta::assert_yaml_snapshot!(input);
        } else {
            unreachable!("cannot get here?")
        }
        Ok(())
    }
}
