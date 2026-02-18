use bsnext_input::startup::{
    RunMode, StartupContext, SystemStart, SystemStartArgs, TopLevelRunMode,
};
use bsnext_input::{Input, InputError};

#[derive(Debug, Clone)]
pub struct RunFromInput {
    input: Input,
    named: Vec<String>,
    run_mode: RunMode,
    top_level_run_mode: TopLevelRunMode,
}

impl RunFromInput {
    pub fn new(
        input: Input,
        named: Vec<String>,
        mode: RunMode,
        top_level_run_mode: TopLevelRunMode,
    ) -> Self {
        Self {
            input,
            named,
            run_mode: mode,
            top_level_run_mode,
        }
    }
}

impl SystemStart for RunFromInput {
    fn input(&self, _ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        Ok(SystemStartArgs::RunOnly {
            input: self.input.clone(),
            named: self.named.clone(),
            run_mode: self.run_mode.clone(),
            top_level_run_mode: self.top_level_run_mode.clone(),
        })
    }
}
