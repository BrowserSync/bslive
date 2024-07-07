#[derive(Debug, Clone)]
pub enum MetaData {
    ServedFile,
    ServedRaw,
    Proxied,
}

impl MetaData {
    pub fn as_header_value(&self) -> &'static str {
        match self {
            MetaData::ServedFile => "MetaData::ServedFile",
            MetaData::ServedRaw => "MetaData::ServedRaw",
            MetaData::Proxied => "MetaData::Proxied",
        }
    }
}
