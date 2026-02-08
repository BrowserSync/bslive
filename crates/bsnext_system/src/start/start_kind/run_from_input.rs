use bsnext_input::startup::{StartupContext, SystemStart, SystemStartArgs};
use bsnext_input::{Input, InputError};

#[derive(Debug, Clone)]
pub struct RunFromInput {
    input: Input,
    named: Vec<String>,
}

impl RunFromInput {
    pub fn new(input: Input, named: Vec<String>) -> Self {
        Self { input, named }
    }
}

impl SystemStart for RunFromInput {
    fn input(&self, _ctx: &StartupContext) -> Result<SystemStartArgs, Box<InputError>> {
        Ok(SystemStartArgs::RunOnly {
            input: self.input.clone(),
            named: self.named.clone(),
        })
    }
}
