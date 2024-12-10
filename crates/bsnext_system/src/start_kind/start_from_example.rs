use crate::start_kind::fs_write_input;
use bsnext_example::Example;
use bsnext_fs_helpers::WriteMode;
use bsnext_input::server_config::ServerIdentity;
use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
use bsnext_input::target::TargetKind;
use bsnext_input::{rand_word, InputError, InputSource, InputSourceKind};

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
        // todo: mimic this for other kinds of startup - basically allow 'port' to be given and respected
        let identity =
            ServerIdentity::from_port_or_named(self.port).map_err(|e| Box::new(e.into()))?;
        let input_source_kind = self.example.into_input(Some(identity));

        let write_mode = if self.force {
            WriteMode::Override
        } else {
            WriteMode::Safe
        };

        let target_dir = if self.temp {
            let temp_dir = std::env::temp_dir();
            let name = self.name.clone();
            let word = name.unwrap_or_else(rand_word);
            let num = rand::random::<f64>();
            let next_dir = temp_dir.join(format!("bslive-{word}-{num}"));
            bsnext_fs_helpers::create_dir(&next_dir, &write_mode)?
        } else if let Some(dir) = &self.dir {
            let next_dir = ctx.cwd.join(dir);
            bsnext_fs_helpers::create_dir(&next_dir, &write_mode)?
        } else {
            ctx.cwd.to_path_buf()
        };

        if !self.write_input {
            tracing::info!("NOT writing input...");
            return match input_source_kind {
                InputSourceKind::Type(input) => Ok(SystemStartArgs::InputOnly { input }),
                InputSourceKind::File { input, .. } => Ok(SystemStartArgs::InputOnly { input }),
            };
        }

        tracing::info!(
            "will write to {} because write_input was true.",
            target_dir.display()
        );

        let (path, input) = match input_source_kind {
            InputSourceKind::Type(input) => {
                let path =
                    fs_write_input(&target_dir, &input, self.target_kind.clone(), &write_mode)
                        .map_err(|e| Box::new(e.into()))?;
                (path, input)
            }
            InputSourceKind::File { src_file, input } => {
                let path = bsnext_fs_helpers::fs_write_input_src(
                    &target_dir,
                    src_file.path(),
                    src_file.content(),
                    &write_mode,
                )
                .map_err(|e| Box::new(e.into()))?;
                (path, input)
            }
        };

        Ok(SystemStartArgs::PathWithInput { path, input })
    }
}
