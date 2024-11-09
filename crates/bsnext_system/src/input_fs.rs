use bsnext_input::{Input, InputCreation, InputError};
use std::path::Path;

pub fn from_input_path<P: AsRef<Path>>(path: P) -> Result<Input, Box<InputError>> {
    match path.as_ref().extension().and_then(|x| x.to_str()) {
        None => Err(Box::new(InputError::MissingExtension(
            path.as_ref().to_owned(),
        ))),
        Some("yml") | Some("yaml") => bsnext_yaml::yaml_fs::YamlFs::from_input_path(path),
        Some("md") | Some("markdown") => bsnext_md::md_fs::MdFs::from_input_path(path),
        Some(other) => Err(Box::new(InputError::UnsupportedExtension(
            other.to_string(),
        ))),
    }
}
