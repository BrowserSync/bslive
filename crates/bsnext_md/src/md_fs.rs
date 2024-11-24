use crate::{nodes_to_input, str_to_nodes, MarkdownError};
use bsnext_input::{Input, InputCreation, InputCtx, InputError};
use std::fs::read_to_string;
use std::path::Path;

pub struct MdFs;

impl InputCreation for MdFs {
    fn from_input_path<P: AsRef<Path>>(path: P, ctx: &InputCtx) -> Result<Input, Box<InputError>> {
        let str = read_to_string(path).map_err(|e| Box::new(e.into()))?;
        let input = md_to_input(&str, ctx)
            .map_err(|e| Box::new(InputError::MarkdownError(e.to_string())))?;
        Ok(input)
    }
    fn from_input_str<P: AsRef<str>>(content: P, ctx: &InputCtx) -> Result<Input, Box<InputError>> {
        let input = md_to_input(content.as_ref(), ctx)
            .map_err(|e| Box::new(InputError::MarkdownError(e.to_string())))?;
        Ok(input)
    }
}

fn md_to_input(input: &str, ctx: &InputCtx) -> Result<Input, MarkdownError> {
    let root = str_to_nodes(input)?;
    nodes_to_input(&root, ctx)
}
