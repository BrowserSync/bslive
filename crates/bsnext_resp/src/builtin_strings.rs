use crate::injector_guard::ByteReplacer;

#[derive(Debug, PartialEq, Hash, Clone, serde::Deserialize, serde::Serialize)]
pub enum BuiltinStrings {
    #[serde(rename = "bslive:connector")]
    Connector,
}

impl ByteReplacer for BuiltinStrings {
    fn apply(&self, body: &'_ str) -> Option<String> {
        match self {
            BuiltinStrings::Connector => {
                let next_body = body.replace(
                    "</body>",
                    format!(
                        "<!-- source: snippet.html-->\
                {}\
                \
                <!-- end: snippet.html-->
                </body>",
                        include_str!("js/snippet.html")
                    )
                    .as_str(),
                );
                Some(next_body)
            }
        }
    }
}
