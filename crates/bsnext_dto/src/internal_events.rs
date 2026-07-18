use crate::StartupError;
use bsnext_input::InputError;
#[derive(Debug)]
pub enum InternalEvents {
    InputError(InputError),
    StartupError(StartupError),
}
