use bsnext_input::yml::YamlError;
use bsnext_input::{BsLiveRulesError, Input, InputCreation, InputCtx, InputError};
use miette::NamedSource;
use std::fs::read_to_string;
use std::path::Path;

pub struct YamlFs;

impl InputCreation for YamlFs {
    fn from_input_path<P: AsRef<Path>>(path: P, _ctx: &InputCtx) -> Result<Input, Box<InputError>> {
        let str = read_to_string(&path).map_err(|e| Box::new(e.into()))?;
        if str.trim().is_empty() {
            return Err(Box::new(InputError::YamlError(YamlError::EmptyError {
                path: path.as_ref().to_string_lossy().to_string(),
            })));
        }
        let output = serde_yaml::from_str::<Input>(str.as_str())
            .map_err(move |e| {
                if let Some(loc) = e.location() {
                    BsLiveRulesError {
                        err_span: (loc.index()..loc.index() + 1).into(),
                        src: NamedSource::new(path.as_ref().to_string_lossy(), str),
                        message: e.to_string(),
                        summary: None,
                    }
                } else {
                    unreachable!("handle later")
                }
            })
            .map_err(|e| Box::new(e.into()))?;
        // todo: don't allow duplicates?.
        Ok(output)
    }

    fn from_input_str<P: AsRef<str>>(
        _content: P,
        _ctx: &InputCtx,
    ) -> Result<Input, Box<InputError>> {
        todo!()
    }
}
