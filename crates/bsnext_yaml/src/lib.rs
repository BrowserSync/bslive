pub mod yaml_fs;
use bsnext_input::Input;

pub fn input_to_str(input: &Input) -> String {
    serde_yaml::to_string(&input).expect("create yaml?")
}
