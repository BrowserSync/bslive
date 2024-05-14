use crate::startup::{StartupContext, SystemStart};
use bsnext_input::paths::from_paths;
use bsnext_input::server_config::Identity;
use bsnext_input::target::TargetKind;
use bsnext_input::{fs_write_input, Input, InputError};
use std::path::PathBuf;

#[derive(Debug)]
pub struct StartFromPaths {
    pub paths: Vec<String>,
    pub write_input: bool,
    pub port: Option<u16>,
}

impl SystemStart for StartFromPaths {
    fn input(&self, ctx: &StartupContext) -> Result<(Input, Option<PathBuf>), InputError> {
        let identity = Identity::from_port_or_named(self.port)?;
        let input = from_paths(&ctx.cwd, &self.paths, identity)?;
        if self.write_input {
            let path = fs_write_input(&ctx.cwd, &input, TargetKind::Yaml)?;

            Ok((input, Some(path)))
        } else {
            Ok((input, None))
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
        };
        let ctx = StartupContext {
            cwd: tmp_dir.path().to_path_buf(),
        };
        let i = v.input(&ctx);
        tmp_dir.close()?;
        let (input, _) = i.unwrap();
        insta::assert_debug_snapshot!(input);
        insta::assert_yaml_snapshot!(input);
        Ok(())
    }
}
