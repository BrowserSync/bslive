use crate::start_kind::start_fs;
use bsnext_example::Example;
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
use bsnext_input::target::TargetKind;
use bsnext_input::{rand_word, InputError};

#[derive(Debug)]
pub struct StartFromExample {
    pub example: Example,
    pub write_input: bool,
    pub target_kind: TargetKind,
    pub port: Option<u16>,
    pub temp: bool,
    pub force: bool,
    pub name: Option<String>,
    pub dir: Option<String>,
}

impl SystemStart for StartFromExample {
    fn input(&self, ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        let identity =
            ServerIdentity::from_port_or_named(self.port).map_err(|e| Box::new(e.into()))?;
        let input = self.example.into_input(identity);
        let name = self.name.clone();
        let write_mode = if self.force {
            start_fs::WriteMode::Override
        } else {
            start_fs::WriteMode::Safe
        };
        let dir = if self.temp {
            let temp_dir = std::env::temp_dir();
            let word = name.unwrap_or_else(rand_word);
            let num = rand::random::<f64>();
            let next_dir = temp_dir.join(format!("bslive-{word}-{num}"));
            start_fs::create_dir(&next_dir, &write_mode)?
        } else if let Some(dir) = &self.dir {
            let next_dir = ctx.cwd.join(dir);
            start_fs::create_dir(&next_dir, &write_mode)?
        } else {
            ctx.cwd.to_path_buf()
        };
        if self.write_input {
            tracing::info!(
                "will write to {} because write_input was true.",
                dir.display()
            );
            let path =
                start_fs::fs_write_input(&dir, &input, self.target_kind.clone(), &write_mode)
                    .map_err(|e| Box::new(e.into()))?;

            Ok(SystemStartArgs::PathWithInput { path, input })
        } else {
            Ok(SystemStartArgs::InputOnly { input })
        }
    }
}
