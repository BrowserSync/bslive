use bsnext_example::Example;
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
use bsnext_input::target::TargetKind;
use bsnext_input::{fs_write_input, rand_word, DirError, InputError};
use std::fs;

#[derive(Debug)]
pub struct StartFromExample {
    pub example: Example,
    pub write_input: bool,
    pub target_kind: TargetKind,
    pub port: Option<u16>,
    pub temp: bool,
    pub name: Option<String>,
}

impl SystemStart for StartFromExample {
    fn input(&self, ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        let identity =
            ServerIdentity::from_port_or_named(self.port).map_err(|e| Box::new(e.into()))?;
        let input = self.example.into_input(identity);
        let name = self.name.clone();
        let dir = if self.temp {
            let temp_dir = std::env::temp_dir();
            let word = name.unwrap_or_else(rand_word);
            let num = rand::random::<f64>();
            let next_dir = temp_dir.join(format!("bslive-{word}-{num}"));
            fs::create_dir_all(&next_dir)
                .map_err(|_e| DirError::CannotCreate {
                    path: next_dir.clone(),
                })
                .and_then(|_pb| {
                    std::env::set_current_dir(&next_dir).map_err(|_e| DirError::CannotMove {
                        path: next_dir.clone(),
                    })
                })
                .map(|_| next_dir.clone())
                .map_err(|e| Box::new(e.into()))?
        } else {
            ctx.cwd.to_path_buf()
        };
        if self.write_input {
            let path = fs_write_input(&dir, &input, self.target_kind.clone())
                .map_err(|e| Box::new(e.into()))?;

            Ok(SystemStartArgs::PathWithInput { path, input })
        } else {
            Ok(SystemStartArgs::InputOnly { input })
        }
    }
}
