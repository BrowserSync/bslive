use crate::input_fs::{from_input_path, ResolvedInputOutcome};
use bsnext_input::startup::{
    RunMode, StartupContext, SystemStart, SystemStartArgs, TopLevelRunMode,
};
use bsnext_input::{Input, InputError};

#[derive(Debug, Clone)]
pub struct RunFromInputPaths {
    input: Input,
    input_paths: Vec<String>,
    named: Vec<String>,
    run_mode: RunMode,
    top_level_run_mode: TopLevelRunMode,
}

impl RunFromInputPaths {
    pub fn new(
        input: Input,
        input_paths: Vec<String>,
        named: Vec<String>,
        mode: RunMode,
        top_level_run_mode: TopLevelRunMode,
    ) -> Self {
        Self {
            input,
            input_paths,
            named,
            run_mode: mode,
            top_level_run_mode,
        }
    }
}

impl SystemStart for RunFromInputPaths {
    fn resolve_input(&self, ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        let next = ResolvedInputOutcome::new(ctx.cwd.to_owned(), &self.input_paths);

        // Here we need to return actual errors, but not error on empty input.
        // For example, giving `-i abc.yaml` SHOULD fail if `abc.yaml` is absent,
        // but it's for no inputs to be present in the run command
        let input_from_file = match next {
            ResolvedInputOutcome::Missing { err, .. } => return Err(err),
            ResolvedInputOutcome::GivenPath { ref absolute, .. } => {
                Some((from_input_path(absolute, &Default::default())?, absolute))
            }
            ResolvedInputOutcome::Auto { ref absolute, .. } => {
                Some((from_input_path(absolute, &Default::default())?, absolute))
            }
            ResolvedInputOutcome::Empty => None,
        };
        let input = match input_from_file {
            None => {
                tracing::debug!("using run_cmd values only");
                self.input.clone()
            }
            Some((mut input_from_file, pb)) => {
                tracing::debug!(?pb, "using run_cmd to extend input from file");
                tracing::debug!(?self.input.run);

                input_from_file.run.extend(self.input.run.clone());
                input_from_file
            }
        };

        Ok(SystemStartArgs::RunOnly {
            input,
            named: self.named.clone(),
            run_mode: self.run_mode.clone(),
            top_level_run_mode: self.top_level_run_mode.clone(),
        })
    }
}
