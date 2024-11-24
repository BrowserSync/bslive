use bsnext_input::{Input, InputWriter};

pub struct YamlWriter;

impl InputWriter for YamlWriter {
    fn input_to_str(&self, input: &Input) -> String {
        input_to_str(input)
    }
}

fn input_to_str(input: &Input) -> String {
    serde_yaml::to_string(&input).expect("create yaml?")
}
